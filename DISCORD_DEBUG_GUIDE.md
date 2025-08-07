# 🔧 Discord Message Reception Debug Guide

## 🚨 Current Issue: Cannot receive Discord messages

Based on your configuration analysis, here are the main issues and solutions:

## ❌ Issues Found

### 1. **Invalid Discord Token**
```toml
discord_token = "YOUR_DISCORD_USER_TOKEN"  # ❌ Placeholder value
```
**Problem**: Using placeholder instead of real Discord token

### 2. **Invalid Channel ID**
```toml
discord_channel_id = ["DISCORD_CHANNEL_ID"]  # ❌ Placeholder value
```
**Problem**: Using placeholder instead of real channel ID

### 3. **TOML Configuration Error**
**Problem**: Invalid TOML syntax (fixed)

## ✅ Solutions

### Step 1: Get Your Discord Token

#### Option A: User Token (Recommended for signal scraping)
1. **Open Discord in browser** (discord.com)
2. **Press F12** to open Developer Tools
3. **Go to Network tab**
4. **Send a message** in any channel
5. **Look for requests** to `discord.com/api/v9/channels/...`
6. **Find the `authorization` header** - this is your token
7. **Copy the token** (starts with `MTI...` or similar)

#### Option B: Bot Token
1. **Go to Discord Developer Portal** (discord.com/developers/applications)
2. **Create New Application** or select existing
3. **Go to Bot section**
4. **Create/Reset Token**
5. **Copy the token** (starts with `Bot `)

### Step 2: Get Channel ID

1. **Enable Developer Mode** in Discord:
   - User Settings → Advanced → Developer Mode ✅
2. **Right-click on target channel**
3. **Select "Copy ID"**
4. **Channel ID looks like**: `1234567890123456789`

### Step 3: Update Configuration

Edit your `config.toml`:

```toml
# Discord Bot for signal scraping
discord_token = "YOUR_ACTUAL_DISCORD_TOKEN_HERE"
discord_channel_id = ["YOUR_ACTUAL_CHANNEL_ID_HERE"]
```

### Step 4: Test Configuration

Run the verification script:
```bash
python verify_config.py
```

### Step 5: Test Message Parsing

Run the test script:
```bash
python test_discord_parsing.py
```

### Step 6: Run with Debug Logging

```bash
RUST_LOG=debug cargo run
```

## 🔍 Debugging Steps

### 1. Check Connection Status
Look for these log messages:
```
✅ Discord Gateway connected
🎯 Discord Gateway READY - Logged in as: [username]
🎯 Target channels to monitor: [channel_ids]
```

### 2. Check Message Reception
Look for these log messages:
```
📨 Message from target channel [channel_id]: [author] - '[content]'
```

### 3. Check Signal Detection
Look for these log messages:
```
🔍 Signal detected in message: '[content]'
✅ Pattern [X] matched address: [address]
✅ Token address validated: [pubkey]
🎯 SIGNAL DETECTED! Token: [pubkey]
```

## 🚨 Common Issues & Solutions

### Issue 1: "Authentication failed"
**Solution**: 
- Check token format (should start with `MTI...` for user token or `Bot ` for bot token)
- Ensure token is not expired
- Verify token has proper permissions

### Issue 2: "No messages received"
**Solutions**:
- Verify channel ID is correct
- Ensure bot/user is in the target channel
- Check if channel has message history permissions

### Issue 3: "Messages received but no signals detected"
**Solutions**:
- Verify message format contains "CA" keyword
- Check if token address is valid Solana pubkey
- Ensure message is not from a bot (bot messages are ignored)

### Issue 4: "Invalid session"
**Solution**: 
- Token may be expired - get new token
- Check if account is banned/limited

## 🧪 Test Your Setup

### Test Message Format
Send this message in your Discord channel:
```
CA: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53
```

### Expected Log Output
```
📨 Message from target channel [channel_id]: [author] - 'CA: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53'
🔍 Signal detected in message: 'ca: i9ie6yj9petkc3mhupa16kkvhvhrkbobasakum5vr53'
✅ Pattern 1 matched address: i9ie6yj9petkc3mhupa16kkvhvhrkbobasakum5vr53
✅ Valid pubkey format: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53
✅ Token address validated: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53
🎯 SIGNAL DETECTED! Token: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53
```

## 📞 Need More Help?

If you're still having issues:

1. **Check the logs** for specific error messages
2. **Verify your Discord token** is valid and not expired
3. **Ensure the bot/user** has access to the target channels
4. **Test with a simple message** format first
5. **Check Discord's status** for any service issues

## 🔧 Advanced Debugging

### Enable Verbose Logging
```bash
RUST_LOG=trace cargo run
```

### Check WebSocket Connection
Look for WebSocket connection logs and heartbeat messages.

### Verify Bot Permissions
If using a bot token, ensure it has:
- Read Message History
- View Channels
- Send Messages (for notifications) 