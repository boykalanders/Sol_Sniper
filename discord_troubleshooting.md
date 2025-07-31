# Discord Message Access Troubleshooting

## Issue: Only Getting Messages from Specific Users

### What I Fixed:

1. **Updated Discord Intents**: Changed from `512` to `33280`
   - `512` = GUILD_MESSAGES (basic message access)
   - `32768` = MESSAGE_CONTENT (full message content access)
   - `33280` = Both combined

2. **Added Debug Logging**: You'll now see:
   - All received messages (debug level)
   - Which messages are being processed
   - Why messages are being ignored

### Run with Debug Logging:

```bash
RUST_LOG=debug cargo run
```

### What to Look For in Logs:

✅ **Good signs:**
```
DEBUG: Received message in channel 123456: UserName - Hello world
INFO: Processing message from UserName in target channel: Hello world
```

❌ **Problem signs:**
```
DEBUG: Ignoring message from non-target channel: 999999
DEBUG: Ignoring bot message from: BotName
```

## Token Type Issues:

### If Using User Token:
- User tokens have access to messages the user can see
- Should work with updated intents
- Make sure the user account has access to the channels

### If Using Bot Token:
- Bot needs to be added to the Discord server
- Bot needs proper permissions in the channels
- Bot token format: `Bot YOUR_BOT_TOKEN`

## Common Issues:

### 1. Wrong Token Format
```toml
# ❌ Wrong (if using bot token)
discord_token = "YOUR_BOT_TOKEN"

# ✅ Correct (if using bot token)  
discord_token = "Bot YOUR_BOT_TOKEN"

# ✅ Correct (if using user token)
discord_token = "YOUR_USER_TOKEN"
```

### 2. Channel Access
- Make sure your account/bot can see the channels
- Check channel permissions
- Verify channel IDs are correct

### 3. Rate Limiting
- Discord may limit message access for new tokens
- Try with a well-established account/bot

## Testing Steps:

1. **Run with debug logging**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Send a test message** in your target channel

3. **Check logs** for:
   - Message reception
   - Channel ID matching
   - Content processing

4. **Verify channel IDs**:
   - Right-click channel → Copy Channel ID
   - Compare with your config

## If Still Not Working:

### Try Bot Token Instead:
1. Go to Discord Developer Portal
2. Create new application → Bot
3. Copy bot token
4. Invite bot to server with permissions:
   - Read Messages
   - Read Message History
   - View Channels

### Update Config:
```toml
discord_token = "Bot YOUR_BOT_TOKEN_HERE"
```

The updated intents should fix the issue, but if you're still only getting specific user messages, it's likely a token permission issue.