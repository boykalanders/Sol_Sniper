# 🔧 Fix "chat not found" Telegram Error

## 🚨 Current Error
```
ERROR snipe::notifier: Telegram API error: {"ok":false,"error_code":400,"description":"Bad Request: chat not found"}
```

## 🔍 Root Cause
Your `config.toml` still has placeholder values instead of real Telegram bot credentials.

## ✅ Quick Fix Steps

### Step 1: Get Your Bot Token
1. **Open Telegram** and message `@BotFather`
2. **Send `/newbot`** and follow instructions
3. **Copy the token** (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### Step 2: Get Your User ID
**Option A: Use the Python script**
```bash
python get_user_id.py
```

**Option B: Manual method**
1. **Start a chat** with your bot
2. **Send any message** (e.g., "hello")
3. **Visit in browser**: `https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates`
4. **Find your ID** in the response (look for `"id": 123456789`)

### Step 3: Update config.toml
Replace the placeholder values in your `config.toml`:

```toml
# BEFORE (placeholder values)
tg_token   = "YOUR_TELEGRAM_BOT_TOKEN"
tg_chat    = "YOUR_CHAT_ID_OR_CHANNEL_ID"
tg_authorized_users = ["YOUR_USER_ID"]

# AFTER (real values)
tg_token   = "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
tg_chat    = "123456789"
tg_authorized_users = ["123456789"]
```

### Step 4: Test the Bot
```bash
cargo run
```

You should see:
```
🤖 Starting Telegram bot...
🤖 Telegram bot started successfully
```

## 🧪 Test Commands
Once running, send these to your bot:
- `/start` - Should show help
- `/status` - Should show bot status

## 🚨 Still Getting Errors?

### Check 1: Bot Token
```bash
curl "https://api.telegram.org/bot<YOUR_TOKEN>/getMe"
```
Should return: `{"ok":true,"result":{...}}`

### Check 2: User ID
```bash
curl "https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates"
```
Should show your messages and user ID.

### Check 3: Chat Access
Make sure you've:
- ✅ Started a chat with your bot
- ✅ Sent at least one message to the bot
- ✅ Used the correct user ID (not username)

## 🔄 Alternative: Disable Telegram Temporarily
If you want to run without Telegram for now, comment out the telegram section in `config.toml`:

```toml
# Telegram & Discord
# tg_token   = "YOUR_TELEGRAM_BOT_TOKEN"
# tg_chat    = "YOUR_CHAT_ID_OR_CHANNEL_ID"
# tg_authorized_users = ["YOUR_USER_ID"]
```

## 📞 Need Help?
1. **Check the exact error** in logs
2. **Verify all values** are correct
3. **Test each component** separately
4. **Restart the application** after changes 