# Telegram Bot Setup Guide - Fix Connection Issues

## 🔧 Step-by-Step Setup

### 1. Create Your Telegram Bot

1. **Open Telegram** and search for `@BotFather`
2. **Send `/newbot`** to BotFather
3. **Choose a name** for your bot (e.g., "My Sniper Bot")
4. **Choose a username** ending with 'bot' (e.g., "my_sniper_bot")
5. **Copy the bot token** that BotFather gives you (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### 2. Get Your User ID

1. **Start a chat** with your new bot
2. **Send any message** to the bot (e.g., "hello")
3. **Visit this URL** in your browser (replace with your bot token):
   ```
   https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates
   ```
4. **Find your user ID** in the response (look for `"id": 123456789`)

### 3. Get Chat ID for Notifications (Optional)

If you want notifications in a channel or group:

1. **Add your bot** to the channel/group
2. **Send a message** in the channel/group
3. **Visit the same URL** as above
4. **Find the chat ID** (for groups it starts with `-`)

### 4. Update Your Configuration

Edit your `config.toml` file:

```toml
# Telegram & Discord
tg_token   = "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"  # Your actual bot token
tg_chat    = "123456789"                              # Your user ID or chat ID
tg_authorized_users = ["123456789"]                   # Your user ID
discord_webhook = "https://discord.com/api/webhooks/..."
```

### 5. Test the Bot

1. **Start your bot**:
   ```bash
   cargo run
   ```

2. **Send commands** to your Telegram bot:
   - `/start` - Should show help message
   - `/status` - Should show bot status
   - `/profit` - Should show profit info

## 🚨 Common Issues & Solutions

### Issue 1: "Bot token is invalid"
- **Solution**: Double-check your bot token from BotFather
- **Make sure**: No extra spaces or characters

### Issue 2: "Unauthorized access"
- **Solution**: Add your user ID to `tg_authorized_users` array
- **Format**: `["123456789"]` (as string, not number)

### Issue 3: "TerminatedByOtherGetUpdates"
- **Solution**: Kill any other instances of your bot
- **Command**: `pkill -f snipe` then restart

### Issue 4: Bot doesn't respond
- **Check**: Bot token is correct
- **Check**: User ID is in authorized list
- **Check**: You started a chat with the bot

## 🔍 Debugging Steps

1. **Check logs** when starting the bot:
   ```bash
   cargo run 2>&1 | grep -i telegram
   ```

2. **Test bot token** manually:
   ```bash
   curl "https://api.telegram.org/bot<YOUR_TOKEN>/getMe"
   ```

3. **Check for running instances**:
   ```bash
   ps aux | grep snipe
   ```

## ✅ Verification Checklist

- [ ] Bot token is valid and not a placeholder
- [ ] User ID is correctly added to `tg_authorized_users`
- [ ] Chat ID is properly formatted (numeric for users, negative for groups)
- [ ] No other bot instances are running
- [ ] You've started a chat with your bot
- [ ] Bot responds to `/start` command

## 🆘 Still Not Working?

If the bot still doesn't connect:

1. **Check the exact error message** in the logs
2. **Verify all configuration values** are correct
3. **Try the helper script** to get your user ID:
   ```bash
   export TELEGRAM_BOT_TOKEN="your_token_here"
   cargo run --bin get_telegram_id
   ```
4. **Restart the application** after making changes 