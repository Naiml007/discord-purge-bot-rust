use crate::types::{TrackedMessage, UserMessages};
use serenity::all::UserId;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

pub struct MessageTracker {
    messages: Arc<RwLock<UserMessages>>,
    retention_period: i64,
}

impl MessageTracker {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(UserMessages::new())),
            retention_period: 48 * 60 * 60 * 1000, // 48 hours in milliseconds
        }
    }

    pub fn get_messages(&self) -> Arc<RwLock<UserMessages>> {
        Arc::clone(&self.messages)
    }

    pub async fn add_message(&self, user_id: UserId, message_id: u64, channel_id: u64, timestamp: i64) {
        let mut messages = self.messages.write().await;
        let user_messages = messages.entry(user_id).or_insert_with(Vec::new);

        // Avoid duplicates
        if user_messages.iter().any(|m| m.message_id == message_id) {
            return;
        }

        user_messages.push(TrackedMessage::new(message_id, channel_id, timestamp));
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
        messages.get(&user_id).cloned().unwrap_or_default()
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
    }

    pub async fn get_stats(&self) -> (usize, usize) {
        let messages = self.messages.read().await;
        let user_count = messages.len();
        let total_messages: usize = messages.values().map(|v| v.len()).sum();
        (user_count, total_messages)
    }

    pub fn start_garbage_collection(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // 1 hour
            loop {
                interval.tick().await;
                self.cleanup().await;
            }
        });
    }
}
