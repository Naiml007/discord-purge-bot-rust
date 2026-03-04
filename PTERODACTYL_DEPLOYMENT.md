# Deploying Rust Discord Bot on Pterodactyl Panel

## Prerequisites

- Pterodactyl panel with Rust egg installed
- FTP/SFTP access to your server
- Discord bot token and client ID

## Method 1: Build Locally, Upload Binary (Recommended)

This method is faster and uses less server resources.

### Step 1: Build Locally

On your local machine with Rust installed:

```bash
cd discord-purge-bot-rust

# Build optimized release binary
cargo build --release

# The binary will be at: target/release/discord_purge_bot
# (or discord_purge_bot.exe on Windows)
```

### Step 2: Upload to Pterodactyl

1. Connect via SFTP to your Pterodactyl server
2. Upload these files to the server root:
   - `target/release/discord_purge_bot` (the binary)
   - `.env` (your environment configuration)

3. Set executable permissions (via SSH or Pterodactyl console):
```bash
chmod +x discord_purge_bot
```

### Step 3: Configure Startup

In Pterodactyl panel:

**Startup Command:**
```bash
./discord_purge_bot
```

**Docker Image:** Use a minimal image like:
- `ghcr.io/parkervcp/yolks:debian` (recommended)
- `ghcr.io/parkervcp/yolks:ubuntu`

### Step 4: Set Environment Variables

In Pterodactyl panel, go to "Startup" tab and add:

```
TOKEN=your_bot_token_here
CLIENT_ID=your_client_id_here
ALLOWED_ROLE_ID=your_role_id_here
```

Or create `.env` file via File Manager:
```env
TOKEN=your_bot_token_here
CLIENT_ID=your_client_id_here
ALLOWED_ROLE_ID=optional_role_id
```

### Step 5: Start the Bot

Click "Start" in Pterodactyl panel. You should see:
```
✅ Logged in as YourBot#1234! Active Tracking System Online.
📜 Starting background history scan...
```

---

## Method 2: Build on Server (Requires Rust Egg)

This method builds directly on the Pterodactyl server.

### Step 1: Upload Source Code

Upload all source files via SFTP:
```
discord-purge-bot-rust/
├── Cargo.toml
├── Cargo.lock (if exists)
├── .env
└── src/
    ├── main.rs
    ├── types.rs
    ├── commands/
    │   ├── mod.rs
    │   └── purge.rs
    └── services/
        ├── mod.rs
        ├── message_tracker.rs
        └── history_scanner.rs
```

### Step 2: Configure Pterodactyl Egg

**If using Generic Rust Egg:**

**Startup Command:**
```bash
if [ -f "discord_purge_bot" ]; then ./discord_purge_bot; else cargo build --release && cp target/release/discord_purge_bot . && ./discord_purge_bot; fi
```

**Or simpler:**
```bash
cargo run --release
```

**Docker Image:**
- `ghcr.io/parkervcp/yolks:rust_latest`

### Step 3: Set Environment Variables

Same as Method 1 - add in Startup tab or `.env` file.

### Step 4: Start Server

First start will compile (takes 5-10 minutes), subsequent starts are instant.

---

## Method 3: Custom Startup Script (Most Flexible)

### Step 1: Create startup.sh

Create `startup.sh` in server root:

```bash
#!/bin/bash

echo "🦀 Starting Rust Discord Bot..."

# Check if binary exists
if [ ! -f "discord_purge_bot" ]; then
    echo "📦 Binary not found. Building from source..."
    cargo build --release
    cp target/release/discord_purge_bot .
    chmod +x discord_purge_bot
fi

# Check for .env file
if [ ! -f ".env" ]; then
    echo "⚠️  Warning: .env file not found!"
    echo "Creating from environment variables..."
    echo "TOKEN=${TOKEN}" > .env
    echo "CLIENT_ID=${CLIENT_ID}" >> .env
    echo "ALLOWED_ROLE_ID=${ALLOWED_ROLE_ID}" >> .env
fi

# Run the bot
echo "🚀 Starting bot..."
./discord_purge_bot
```

### Step 2: Make Executable

```bash
chmod +x startup.sh
```

### Step 3: Pterodactyl Startup Command

```bash
bash startup.sh
```

---

## Pterodactyl Egg Configuration

If you need to create a custom egg, here's the configuration:

### Egg JSON (egg-rust-discord-bot.json)

```json
{
    "meta": {
        "version": "PTDL_v2",
        "update_url": null
    },
    "exported_at": "2024-01-01T00:00:00+00:00",
    "name": "Rust Discord Bot",
    "author": "admin@example.com",
    "description": "Rust-based Discord bot with low resource usage",
    "features": null,
    "docker_images": {
        "ghcr.io/parkervcp/yolks:rust_latest": "ghcr.io/parkervcp/yolks:rust_latest"
    },
    "file_denylist": [],
    "startup": "./discord_purge_bot",
    "config": {
        "files": "{}",
        "startup": "{\r\n    \"done\": \"Active Tracking System Online\"\r\n}",
        "logs": "{}",
        "stop": "^C"
    },
    "scripts": {
        "installation": {
            "script": "#!/bin/bash\r\napt update\r\napt install -y curl build-essential\r\ncurl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y\r\nsource $HOME/.cargo/env\r\ncd /mnt/server\r\ncargo build --release\r\ncp target/release/discord_purge_bot .\r\nchmod +x discord_purge_bot",
            "container": "rust:latest",
            "entrypoint": "bash"
        }
    },
    "variables": [
        {
            "name": "Discord Token",
            "description": "Discord bot token",
            "env_variable": "TOKEN",
            "default_value": "",
            "user_viewable": true,
            "user_editable": true,
            "rules": "required|string",
            "field_type": "text"
        },
        {
            "name": "Client ID",
            "description": "Discord application client ID",
            "env_variable": "CLIENT_ID",
            "default_value": "",
            "user_viewable": true,
            "user_editable": true,
            "rules": "required|string",
            "field_type": "text"
        },
        {
            "name": "Allowed Role ID",
            "description": "Optional role ID for command access",
            "env_variable": "ALLOWED_ROLE_ID",
            "default_value": "",
            "user_viewable": true,
            "user_editable": true,
            "rules": "nullable|string",
            "field_type": "text"
        }
    ]
}
```

---

## Resource Allocation

### Recommended Pterodactyl Limits

```
Memory: 128 MB (minimum) - 256 MB (recommended)
CPU: 50% (minimum) - 100% (recommended)
Disk: 50 MB (binary only) - 500 MB (with source)
```

The bot is extremely lightweight:
- Idle: ~20-30 MB RAM, <0.1% CPU
- Active: ~40-50 MB RAM, <1% CPU
- Startup scan: ~50-80 MB RAM, 2-10% CPU (temporary)

---

## Troubleshooting

### "Permission denied" Error

```bash
chmod +x discord_purge_bot
```

### "Cannot find binary"

Make sure you're running from the correct directory:
```bash
ls -la
# Should show discord_purge_bot file
```

### Build Fails on Server

If server has limited resources, build locally and upload binary (Method 1).

### Bot Crashes on Startup

Check logs in Pterodactyl console:
```bash
# Common issues:
# 1. Missing TOKEN in environment
# 2. Invalid token
# 3. Missing intents in Discord Developer Portal
```

### Environment Variables Not Loading

Create `.env` file manually:
```bash
nano .env
```

Add:
```env
TOKEN=your_token
CLIENT_ID=your_id
ALLOWED_ROLE_ID=your_role_id
```

Save with `Ctrl+X`, `Y`, `Enter`

### Bot Restarts Constantly

Check if:
1. Token is valid
2. Bot has proper intents enabled (GUILDS, GUILD_MESSAGES)
3. No port conflicts (bot doesn't need ports)

---

## Auto-Restart Configuration

In Pterodactyl panel, enable auto-restart:

1. Go to "Startup" tab
2. Enable "Auto Restart"
3. Set restart delay: 10 seconds

This ensures the bot restarts if it crashes.

---

## Updating the Bot

### Method 1 (Binary Upload):
1. Build new version locally: `cargo build --release`
2. Stop bot in Pterodactyl
3. Upload new binary via SFTP
4. Start bot

### Method 2 (Source on Server):
1. Upload new source files via SFTP
2. Stop bot
3. Run: `cargo build --release`
4. Run: `cp target/release/discord_purge_bot .`
5. Start bot

---

## Monitoring

### Check Bot Status

In Pterodactyl console:
```bash
# View logs
tail -f /path/to/logs

# Check memory usage
ps aux | grep discord_purge_bot

# Check if running
pgrep -f discord_purge_bot
```

### Performance Monitoring

The bot logs important events:
- Startup confirmation
- History scan completion
- Garbage collection runs
- Command executions
- Errors

---

## Security Best Practices

1. **Never commit .env file** to version control
2. **Use environment variables** in Pterodactyl instead of .env file
3. **Restrict file permissions**: `chmod 600 .env`
4. **Use ALLOWED_ROLE_ID** to restrict command access
5. **Regular updates**: Keep dependencies updated

---

## Quick Start Checklist

- [ ] Build binary locally or prepare source code
- [ ] Upload files to Pterodactyl server
- [ ] Set environment variables (TOKEN, CLIENT_ID)
- [ ] Configure startup command
- [ ] Set executable permissions
- [ ] Enable Discord intents (GUILDS, GUILD_MESSAGES)
- [ ] Start server
- [ ] Verify bot is online in Discord
- [ ] Test `/purge` command

---

## Support

If you encounter issues:

1. Check Pterodactyl console logs
2. Verify Discord Developer Portal settings
3. Ensure bot has proper permissions in Discord server
4. Check environment variables are set correctly
5. Verify binary is executable: `ls -la discord_purge_bot`

## Performance Comparison

| Metric | Node.js Bot | Rust Bot |
|--------|-------------|----------|
| Memory | 150-200 MB | 20-50 MB |
| CPU (idle) | 1-3% | <0.1% |
| Startup | 2-3s | <1s |
| Pterodactyl Cost | Higher tier | Lowest tier |

The Rust version can run on the cheapest Pterodactyl plans!
