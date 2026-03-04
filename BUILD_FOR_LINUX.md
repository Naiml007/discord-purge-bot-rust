# Building Linux Binary on Windows

## Method 1: Cross-Compile with cross (Easiest)

### Step 1: Install cross
```bash
cargo install cross
```

### Step 2: Build for Linux
```bash
cd discord-purge-bot-rust

# For x86_64 Linux (most common)
cross build --release --target x86_64-unknown-linux-gnu

# The Linux binary will be at:
# target/x86_64-unknown-linux-gnu/release/discord_purge_bot
```

### Step 3: Upload to Pterodactyl
Upload the file from:
```
target/x86_64-unknown-linux-gnu/release/discord_purge_bot
```

This is a native Linux binary that will run on your Pterodactyl server!

---

## Method 2: Use WSL (Windows Subsystem for Linux)

### Step 1: Install WSL
```powershell
wsl --install
```

### Step 2: Install Rust in WSL
```bash
# Open WSL terminal
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Step 3: Copy project to WSL and build
```bash
# In WSL terminal
cd /mnt/c/Coding/bun-discord-bot/discord-purge-bot-rust
cargo build --release

# Binary will be at: target/release/discord_purge_bot (Linux binary)
```

### Step 4: Upload to Pterodactyl
The binary at `target/release/discord_purge_bot` is now a Linux binary.

---

## Method 3: Build Directly on Pterodactyl (If you have enough resources)

If your server can handle it (needs ~1GB RAM temporarily):

### Step 1: Upload source code via SFTP
Upload these files:
```
Cargo.toml
.env
src/ (entire folder with all .rs files)
```

### Step 2: Use Rust Docker Image
In Pterodactyl, set Docker image to:
```
ghcr.io/parkervcp/yolks:rust_latest
```

### Step 3: Startup Command
```bash
if [ -f "discord_purge_bot" ]; then 
    ./discord_purge_bot
else 
    cargo build --release && cp target/release/discord_purge_bot . && ./discord_purge_bot
fi
```

First start will compile (5-10 minutes), then it saves the binary for instant future starts.

---

## Recommended Approach

**For limited CPU servers: Use Method 1 (cross)**

1. Install cross: `cargo install cross`
2. Build: `cross build --release --target x86_64-unknown-linux-gnu`
3. Upload: `target/x86_64-unknown-linux-gnu/release/discord_purge_bot`
4. Done!

---

## Quick Comparison

| Method | Build Time | Server CPU | Complexity |
|--------|------------|------------|------------|
| cross | 2-3 min | None | Easy |
| WSL | 2-3 min | None | Medium |
| On Server | 5-10 min | High | Easy |

---

## Troubleshooting cross

### If cross fails to install:
```bash
# Make sure Docker Desktop is installed and running
# Download from: https://www.docker.com/products/docker-desktop/

# Then retry:
cargo install cross
```

### If cross build fails:
```bash
# Try musl target (more compatible):
cross build --release --target x86_64-unknown-linux-musl
```

---

## File Sizes

- Windows .exe: ~5-10 MB
- Linux binary: ~5-10 MB
- Source code: ~50 KB
- Full target folder: 100-500 MB (don't upload this!)

---

## After Building

Upload to Pterodactyl:
1. The Linux binary (5-10 MB)
2. .env file

Set startup command:
```bash
chmod +x discord_purge_bot && ./discord_purge_bot
```

That's it! The bot will run on Linux.
