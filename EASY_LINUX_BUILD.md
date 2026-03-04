# Easy Ways to Get Linux Binary

You're on Windows and need a Linux binary for Pterodactyl. Here are the easiest options:

## Option 1: Use GitHub Actions (Recommended - Free & Automatic)

### Steps:
1. Create a GitHub repository
2. Push your code to GitHub
3. GitHub will automatically build the Linux binary
4. Download it from the "Actions" tab

### How to use:
```bash
# Initialize git (if not already)
cd discord-purge-bot-rust
git init
git add .
git commit -m "Initial commit"

# Create repo on GitHub, then:
git remote add origin https://github.com/YOUR_USERNAME/YOUR_REPO.git
git push -u origin main
```

Then:
1. Go to your GitHub repo
2. Click "Actions" tab
3. Click "Build Linux Binary"
4. Download the artifact (discord_purge_bot-linux)

---

## Option 2: Install WSL (5 minutes)

### Steps:
```powershell
# Run in PowerShell as Administrator
wsl --install

# Restart your computer

# After restart, open "Ubuntu" from Start Menu
# Install Rust in WSL:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Navigate to your project:
cd /mnt/c/Coding/bun-discord-bot/discord-purge-bot-rust

# Build for Linux:
cargo build --release

# Binary is at: target/release/discord_purge_bot (Linux binary!)
```

---

## Option 3: Use Online Rust Playground (For small projects)

Not ideal for this project, but worth mentioning.

---

## Option 4: Build on Pterodactyl Server (If you have resources)

### Requirements:
- At least 1GB RAM allocated
- Rust docker image

### Steps:

1. **Upload source code via SFTP:**
   - Upload entire `src/` folder
   - Upload `Cargo.toml`
   - Upload `.env`

2. **In Pterodactyl Panel:**
   - Docker Image: `ghcr.io/parkervcp/yolks:rust_latest`
   - Startup Command:
   ```bash
   if [ ! -f "discord_purge_bot" ]; then cargo build --release && cp target/release/discord_purge_bot .; fi && ./discord_purge_bot
   ```

3. **Start server** - First start will compile (5-10 min), then saves binary

4. **After first build**, you can switch to minimal image:
   - Docker Image: `ghcr.io/parkervcp/yolks:debian`
   - Startup Command: `./discord_purge_bot`

---

## Option 5: Ask a Friend with Linux

Send them:
- The entire `discord-purge-bot-rust` folder
- Ask them to run: `cargo build --release`
- They send back: `target/release/discord_purge_bot`

---

## Recommended: WSL (Option 2)

**Why?**
- One-time 5-minute setup
- Build Linux binaries anytime
- No external dependencies
- Free

**Quick Install:**
```powershell
# PowerShell as Admin
wsl --install
# Restart PC
# Open Ubuntu from Start Menu
# Done!
```

---

## What You Need to Upload to Pterodactyl

**If you have Linux binary:**
- `discord_purge_bot` (5-10 MB)
- `.env` file

**If building on server:**
- `src/` folder (all .rs files)
- `Cargo.toml`
- `.env` file

---

## Quick Decision Guide

| Your Situation | Best Option |
|----------------|-------------|
| Want it now, have GitHub | Option 1 (GitHub Actions) |
| Want to build locally | Option 2 (WSL) |
| Server has 1GB+ RAM | Option 4 (Build on server) |
| Need help | Ask me! |

---

## After You Get Linux Binary

1. Upload to Pterodactyl via SFTP
2. In console: `chmod +x discord_purge_bot`
3. Startup command: `./discord_purge_bot`
4. Docker image: `ghcr.io/parkervcp/yolks:debian`
5. Start server ✅

---

## Need Help?

Let me know which option you want to try and I'll guide you through it!
