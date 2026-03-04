use crate::types::{TrackedMessage, UserMessages};
use serenity::all::UserId;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

pub struct MessageTracker {
    messages: Arc<RwLock<UserMessages>>,
    retention_period: i64,
    max_messages_per_user: usize,
    max_total_messages: usize,
    scanned_message_ids: Arc<RwLock<HashSet<u64>>>, // Track messages from history scan
}

impl MessageTracker {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(UserMessages::new())),
            retention_period: 48 * 60 * 60 * 1000, // 48 hours in milliseconds
            max_messages_per_user: 1000, // Limit per user to prevent unbounded growth
            max_total_messages: 50000, // Global limit across all users
            scanned_message_ids: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn get_messages(&self) -> Arc<RwLock<UserMessages>> {
        Arc::clone(&self.messages)
    }

    pub async fn add_message(&self, user_id: UserId, message_id: u64, channel_id: u64, timestamp: i64) {
        self.add_message_internal(user_id, message_id, channel_id, timestamp, false).await;
    }
    
    pub async fn add_message_from_scan(&self, user_id: UserId, message_id: u64, channel_id: u64, timestamp: i64) {
        self.add_message_internal(user_id, message_id, channel_id, timestamp, true).await;
    }
    
    async fn add_message_internal(&self, user_id: UserId, message_id: u64, channel_id: u64, timestamp: i64, from_scan: bool) {
        // Fast check for scanned messages to avoid duplicate processing
        if from_scan {
            let scanned = self.scanned_message_ids.read().await;
            if scanned.contains(&message_id) {
                return; // Already scanned this message
            }
        }
        
        let mut messages = self.messages.write().await;
        
        // Check global message limit
        let total_messages: usize = messages.values().map(|v| v.len()).sum();
        if total_messages >= self.max_total_messages {
            debug!("Global message limit reached ({}), skipping add", self.max_total_messages);
            return;
        }
        
        let user_messages = messages.entry(user_id).or_insert_with(Vec::new);

        // Avoid duplicates
        if user_messages.iter().any(|m| m.message_id == message_id) {
            if from_scan {
                // Mark as scanned even if duplicate
                drop(messages);
                let mut scanned = self.scanned_message_ids.write().await;
                scanned.insert(message_id);
            }
            return;
        }

        // Enforce per-user limit with LRU eviction
        if user_messages.len() >= self.max_messages_per_user {
            user_messages.remove(0); // Remove oldest message
            debug!("Per-user limit reached for {}, evicting oldest message", user_id);
        }

        user_messages.push(TrackedMessage::new(message_id, channel_id, timestamp));
        
        // Track scanned messages
        if from_scan {
            drop(messages);
            let mut scanned = self.scanned_message_ids.write().await;
            scanned.insert(message_id);
        }
    }

    pub async fn remove_message(&self, message_id: u64) {
        let mut messages = self.messages.write().await;
        
        let mut user_to_remove = None;
        
        for (user_id, user_messages) in messages.iter_mut() {
            if let Some(index) = user_messages.iter().position(|m| m.message_id == message_id) {
                user_messages.remove(index);
                if user_messages.is_empty() {
                    user_to_remove = Some(*user_id);
                }
                break;
            }
        }
        
        if let Some(user_id) = user_to_remove {
            messages.remove(&user_id);
        }
    }

    pub async fn remove_messages(&self, message_ids: &[u64]) {
        let mut messages = self.messages.write().await;
        let ids_set: std::collections::HashSet<u64> = message_ids.iter().copied().collect();

        messages.retain(|_, user_messages| {
            user_messages.retain(|m| !ids_set.contains(&m.message_id));
            !user_messages.is_empty()
        });

        debug!("Bulk delete: removed {} messages from tracker", message_ids.len());
    }

    pub async fn get_user_messages(&self, user_id: UserId) -> Vec<TrackedMessage> {
        let messages = self.messages.read().await;
        // Clone only the specific user's messages, not the entire HashMap
        messages.get(&user_id).cloned().unwrap_or_default()
    }
    
    pub async fn get_user_message_ids(&self, user_id: UserId) -> Vec<(u64, u64)> {
        let messages = self.messages.read().await;
        // Return only IDs to avoid cloning full TrackedMessage structs
        messages.get(&user_id)
            .map(|msgs| msgs.iter().map(|m| (m.message_id, m.channel_id)).collect())
            .unwrap_or_default()
    }

    pub async fn cleanup(&self) {
        let now = chrono::Utc::now().timestamp_millis();
        let mut messages = self.messages.write().await;
        let mut total_removed = 0;

        messages.retain(|_, user_messages| {
            let original_len = user_messages.len();
            user_messages.retain(|m| (now - m.timestamp) < self.retention_period);
            total_removed += original_len - user_messages.len();
            !user_messages.is_empty()
        });

        if total_removed > 0 {
            info!("Garbage collection: removed {} expired messages", total_removed);
        }
        
        // Clean up scanned message IDs to prevent unbounded growth
        drop(messages);
        let mut scanned = self.scanned_message_ids.write().await;
        let scanned_count = scanned.len();
        if scanned_count > 10000 {
            scanned.clear();
            info!("Cleared {} scanned message IDs to prevent memory bloat", scanned_count);
        }
    }

    pub async fn get_stats(&self) -> (usize, usize) {
        let messages = self.messages.read().await;
        let user_count = messages.len();
        let total_messages: usize = messages.values().map(|v| v.len()).sum();
        (user_count, total_messages)
    }

    pub fn start_garbage_collection(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1800)); // 30 minutes (reduced from 1 hour)
            loop {
                interval.tick().await;
                self.cleanup().await;
                
                // Log memory stats after cleanup
                let (user_count, total_messages) = self.get_stats().await;
                info!("Memory stats: {} users, {} total messages tracked", user_count, total_messages);
            }
        });
    }
}
