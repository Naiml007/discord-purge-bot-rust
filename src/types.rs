use serenity::all::UserId;

#[derive(Debug, Clone)]
pub struct TrackedMessage {
    pub message_id: u64,
    pub channel_id: u64,
    pub timestamp: i64,
}

impl TrackedMessage {
    pub fn new(message_id: u64, channel_id: u64, timestamp: i64) -> Self {
        Self {
            message_id,
            channel_id,
            timestamp,
        }
    }
}

pub type UserMessages = std::collections::HashMap<UserId, Vec<TrackedMessage>>;
