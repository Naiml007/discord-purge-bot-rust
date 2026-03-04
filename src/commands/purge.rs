use crate::services::message_tracker::MessageTracker;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse, MessageId, Permissions, ResolvedOption, ResolvedValue, RoleId,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::warn;

pub fn register() -> CreateCommand {
    CreateCommand::new("purge")
        .description("Globally purge messages from a specific user (Last 48 hours)")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "target",
                "The user whose messages to delete",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "amount",
                "Number of messages to delete (Optional)",
            )
            .min_int_value(1)
            .required(false),
        )
        .default_member_permissions(Permissions::MANAGE_MESSAGES)
}

pub async fn execute(
    ctx: &Context,
    interaction: &CommandInteraction,
    tracker: Arc<MessageTracker>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check role permission if configured
    if let Ok(allowed_role_id) = std::env::var("ALLOWED_ROLE_ID") {
        if let Ok(role_id) = allowed_role_id.parse::<u64>() {
            let member = interaction.member.as_ref().ok_or("No member found")?;
            if !member.roles.contains(&RoleId::new(role_id)) {
                let response = CreateInteractionResponseMessage::new()
                    .content("❌ You do not have permission to use this command!")
                    .ephemeral(true);
                interaction
                    .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                    .await?;
                return Ok(());
            }
        }
    }

    // Check bot permissions
    if let Some(guild_id) = interaction.guild_id {
        let current_user_id = ctx.cache.current_user().id;
        let bot_member = guild_id.member(&ctx.http, current_user_id).await?;
        
        // Get guild permissions for the bot
        let permissions = guild_id
            .to_partial_guild(&ctx.http)
            .await?
            .member_permissions(&bot_member);
            
        if !permissions.contains(Permissions::MANAGE_MESSAGES) {
            let response = CreateInteractionResponseMessage::new()
                .content("❌ I do not have Manage Messages permission!")
                .ephemeral(true);
            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await?;
            return Ok(());
        }
    }

    // Defer reply
    let response = CreateInteractionResponseMessage::new().ephemeral(true);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Defer(response))
        .await?;

    // Parse options
    let options = &interaction.data.options();
    let target_user = match options.iter().find(|opt| opt.name == "target") {
        Some(ResolvedOption {
            value: ResolvedValue::User(user, _),
            ..
        }) => user,
        _ => {
            interaction
                .edit_response(&ctx.http, EditInteractionResponse::new().content("❌ Invalid user"))
                .await?;
            return Ok(());
        }
    };

    let amount = options
        .iter()
        .find(|opt| opt.name == "amount")
        .and_then(|opt| {
            if let ResolvedValue::Integer(val) = opt.value {
                Some(val as usize)
            } else {
                None
            }
        });

    // Get tracked messages
    let mut tracked_messages = tracker.get_user_messages(target_user.id).await;

    if let Some(limit) = amount {
        let start = tracked_messages.len().saturating_sub(limit);
        tracked_messages = tracked_messages[start..].to_vec();
    }

    if tracked_messages.is_empty() {
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .content(format!("✅ No active messages found for {} in the last 48 hours.", target_user.tag())),
            )
            .await?;
        return Ok(());
    }

    // Group by channel
    let mut messages_by_channel: HashMap<u64, Vec<u64>> = HashMap::new();
    for msg in tracked_messages {
        messages_by_channel
            .entry(msg.channel_id)
            .or_insert_with(Vec::new)
            .push(msg.message_id);
    }

    let total_channels = messages_by_channel.len();
    let mut channels_processed = 0;
    let mut total_deleted = 0;

    // Delete messages
    for (channel_id, message_ids) in messages_by_channel {
        match delete_messages_in_channel(ctx, channel_id, message_ids).await {
            Ok(deleted) => {
                total_deleted += deleted;
                channels_processed += 1;
            }
            Err(e) => {
                warn!("Failed to delete in channel {}: {:?}", channel_id, e);
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content(format!(
                "✅ Global Purge Complete for {}.\n🗑️ **{}** messages deleted.\n📂 **{}/{}** active channels processed.",
                target_user.tag(),
                total_deleted,
                channels_processed,
                total_channels
            )),
        )
        .await?;

    Ok(())
}

async fn delete_messages_in_channel(
    ctx: &Context,
    channel_id: u64,
    message_ids: Vec<u64>,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let channel = ctx.http.get_channel(channel_id.into()).await?;
    let mut deleted = 0;

    // Process in chunks of 100
    for chunk in message_ids.chunks(100) {
        let msg_ids: Vec<MessageId> = chunk.iter().map(|&id| MessageId::new(id)).collect();

        match channel.id().delete_messages(&ctx.http, msg_ids).await {
            Ok(_) => deleted += chunk.len(),
            Err(e) => {
                // Ignore "Unknown Message" errors
                if e.to_string().contains("10008") {
                    warn!("Some messages already deleted in channel {}", channel_id);
                } else {
                    return Err(Box::new(e));
                }
            }
        }
    }

    Ok(deleted)
}
