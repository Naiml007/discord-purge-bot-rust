# ✅ Linux Binary Ready - Upload to Pterodactyl

## Your Linux Binary is Built!

**Location:** `discord-purge-bot-rust/target/release/discord_purge_bot`
**Size:** 9.4 MB
**Platform:** Linux x86_64

## Step-by-Step Upload Guide

### Step 1: Prepare Files

You need to upload only 2 files:

1. **Binary:** `target/release/discord_purge_bot` (9.4 MB)
2. **Config:** `.env` file with your bot credentials

### Step 2: Connect to Pterodactyl via SFTP

Use any SFTP client (FileZilla, WinSCP, or Pterodactyl's built-in file manager):

**SFTP Details:**
- Host: Your Pterodactyl server address
- Port: Usually 2022
- Username: From Pterodactyl panel
- Password: From Pterodactyl panel

### Step 3: Upload Files

1. Upload `discord_purge_bot` to server root
2. Create or upload `.env` file with:
```env
TOKEN=your_bot_token_here
CLIENT_ID=your_client_id_here
ALLOWED_ROLE_ID=optional_role_id
```

### Step 4: Set Permissions

In Pterodactyl console, run:
```bash
chmod +x discord_purge_bot
```

### Step 5: Configure Pterodactyl

**Docker Image:**
```
ghcr.io/parkervcp/yolks:debian
```

**Startup Command:**
```bash
./discord_purge_bot
```

**Environment Variables** (Optional - if not using .env file):
- `TOKEN` = your_bot_token
- `CLIENT_ID` = your_client_id
- `ALLOWED_ROLE_ID` = your_role_id (optional)

### Step 6: Resource Allocation

**Recommended Settings:**
- Memory: 128 MB minimum (256 MB recommended)
- CPU: 50% minimum
- Disk: 50 MB

### Step 7: Start the Bot

Click "Start" in Pterodactyl panel.

**Expected Output:**
```
✅ Logged in as YourBot#1234! Active Tracking System Online.
📜 Starting background history scan...
```

## Alternative: Use Pterodactyl File Manager

If you don't have SFTP access:

1. Go to Pterodactyl panel → Files
2. Click "Upload" button
3. Upload `discord_purge_bot` file
4. Create new file named `.env`
5. Add your credentials
6. In console: `chmod +x discord_purge_bot`
7. Start server

## Troubleshooting

### "Permission denied"
```bash
chmod +x discord_purge_bot
```

### "No such file or directory"
Make sure you're in the correct directory:
```bash
ls -la
# Should show discord_purge_bot file
```

### Bot doesn't start
Check:
1. TOKEN is valid in .env
2. Discord intents are enabled (GUILDS, GUILD_MESSAGES)
3. File has execute permissions

### "Cannot execute binary file"
Make sure you uploaded the Linux binary from:
`target/release/discord_purge_bot` (NOT the .exe from Windows)

## Performance Expectations

Once running, you should see:
- Memory usage: 20-50 MB
- CPU usage: < 0.1% when idle
- Instant startup (< 1 second)
- No compilation needed

## Testing the Bot

1. Invite bot to your Discord server
2. Wait 1-2 minutes for commands to register
3. Type `/purge` - command should appear
4. Test: `/purge target:@User`

## Success Indicators

✅ Bot shows online in Discord
✅ Console shows "Active Tracking System Online"
✅ `/purge` command appears in Discord
✅ Memory usage < 50 MB
✅ No errors in console

## Next Steps

- Monitor bot performance in Pterodactyl
- Test purge command with a test user
- Check logs for any issues
- Enjoy your super-efficient Rust bot!

## Comparison

| Metric | TypeScript Bot | Rust Bot |
|--------|----------------|----------|
| Binary Size | ~50+ MB | 9.4 MB |
| Memory | 150-200 MB | 20-50 MB |
| CPU (idle) | 1-3% | < 0.1% |
| Startup | 2-3s | < 1s |
| Dependencies | Node.js + modules | None |

## Need Help?

If you encounter issues:
1. Check Pterodactyl console logs
2. Verify .env file has correct credentials
3. Ensure Discord intents are enabled
4. Check file permissions: `ls -la discord_purge_bot`

---

**You're all set!** Your Rust Discord bot is ready to deploy. 🦀🚀
