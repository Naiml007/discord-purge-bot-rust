use crate::services::message_tracker::MessageTracker;
use serenity::all::{ChannelId, Context, GetMessages, GuildId};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub struct HistoryScanner;

impl HistoryScanner {
    const RETENTION_PERIOD: i64 = 48 * 60 * 60 * 1000; // 48 hours in ms
    const BATCH_DELAY: Duration = Duration::from_secs(1);
    const CHANNEL_DELAY: Duration = Duration::from_secs(2);
    const MAX_MESSAGES_PER_CHANNEL: usize = 500;

    pub async fn start(ctx: Context, tracker: Arc<MessageTracker>) {
        tokio::spawn(async move {
            info!("📜 Starting background history scan...");
            let mut total_tracked = 0;

            let guilds = ctx.cache.guilds();
            
            for guild_id in guilds {
                if let Err(e) = Self::scan_guild(&ctx, guild_id, &tracker, &mut total_tracked).await {
                    error!("Failed to scan guild {}: {:?}", guild_id, e);
                }
            }

            info!("✅ History scan complete. Tracked {} messages", total_tracked);
        });
    }

    async fn scan_guild(
        ctx: &Context,
        guild_id: GuildId,
        tracker: &Arc<MessageTracker>,
        total_tracked: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let guild = match guild_id.to_partial_guild(&ctx.http).await {
            Ok(g) => g,
            Err(e) => {
                warn!("Could not fetch guild {}: {:?}", guild_id, e);
                return Ok(());
            }
        };

        let channels = guild.channels(&ctx.http).await?;

        for (channel_id, channel) in channels {
            if !channel.is_text_based() {
                continue;
            }

            match Self::scan_channel(ctx, channel_id, tracker).await {
                Ok(count) => {
                    *total_tracked += count;
                }
                Err(e) => {
                    warn!("Error scanning channel {}: {:?}", channel_id, e);
                }
            }

            sleep(Self::CHANNEL_DELAY).await;
        }

        Ok(())
    }

    async fn scan_channel(
        ctx: &Context,
        channel_id: ChannelId,
        tracker: &Arc<MessageTracker>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp_millis();
        let mut last_id = None;
        let mut count = 0;

        loop {
            if count >= Self::MAX_MESSAGES_PER_CHANNEL {
                break;
            }

            let builder = GetMessages::new().limit(50);
            let builder = if let Some(id) = last_id {
                builder.before(id)
            } else {
                builder
            };

            let messages = match channel_id.messages(&ctx.http, builder).await {
                Ok(msgs) => msgs,
                Err(_) => break, // No permission or channel gone
            };

            if messages.is_empty() {
                break;
            }

            for msg in messages {
                let msg_timestamp = msg.timestamp.timestamp_millis();
                
                if now - msg_timestamp > Self::RETENTION_PERIOD {
                    return Ok(count); // Stop if too old
                }

                if !msg.author.bot {
                    tracker.add_message(msg.author.id, msg.id.get(), msg.channel_id.get(), msg_timestamp).await;
                    count += 1;
                }

                last_id = Some(msg.id);
            }

            sleep(Self::BATCH_DELAY).await;
        }

        Ok(count)
    }
}
