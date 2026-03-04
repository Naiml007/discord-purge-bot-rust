mod commands;
mod services;
mod types;

use serenity::all::{
    Client, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
    GatewayIntents, Interaction, Message, MessageId, Ready,
};
use services::history_scanner::HistoryScanner;
use services::message_tracker::MessageTracker;
use std::sync::Arc;
use tracing::{error, info};

struct Handler {
    tracker: Arc<MessageTracker>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("✅ Logged in as {}! Active Tracking System Online.", ready.user.tag());

        // Register commands globally
        if let Err(e) = serenity::all::Command::create_global_command(&ctx.http, commands::purge::register())
            .await
        {
            error!("Failed to register commands: {:?}", e);
        }

        // Start history scanner
        HistoryScanner::start(ctx, Arc::clone(&self.tracker)).await;
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        // Ignore bots and DMs
        if msg.author.bot || msg.guild_id.is_none() {
            return;
        }

        let timestamp = msg.timestamp.timestamp_millis();
        self.tracker
            .add_message(msg.author.id, msg.id.get(), msg.channel_id.get(), timestamp)
            .await;
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        _channel_id: serenity::all::ChannelId,
        deleted_message_id: MessageId,
        _guild_id: Option<serenity::all::GuildId>,
    ) {
        self.tracker.remove_message(deleted_message_id.get()).await;
    }

    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        _channel_id: serenity::all::ChannelId,
        multiple_deleted_messages_ids: Vec<MessageId>,
        _guild_id: Option<serenity::all::GuildId>,
    ) {
        let ids: Vec<u64> = multiple_deleted_messages_ids.iter().map(|id| id.get()).collect();
        self.tracker.remove_messages(&ids).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let result = match command.data.name.as_str() {
                "purge" => commands::purge::execute(&ctx, &command, Arc::clone(&self.tracker)).await,
                _ => Ok(()),
            };

            if let Err(e) = result {
                error!("Error executing command: {:?}", e);
                let response = CreateInteractionResponseMessage::new()
                    .content("There was an error while executing this command!")
                    .ephemeral(true);
                let _ = command
                    .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                    .await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN").expect("Missing TOKEN in environment");

    // Initialize tracker
    let tracker = Arc::new(MessageTracker::new());
    
    // Start garbage collection
    Arc::clone(&tracker).start_garbage_collection();

    // Configure intents
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;

    // Create client
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { tracker })
        .await
        .expect("Error creating client");

    // Start client
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
