# 📢 Telegram Channel Setup Guide

## 🎯 Overview
This guide helps you set up your Telegram bot to send notifications to a dedicated channel instead of your personal chat.

## 🔧 Step-by-Step Setup

### Step 1: Create a Telegram Channel

1. **Open Telegram** and tap the menu (☰)
2. **Tap "New Channel"**
3. **Choose a name** (e.g., "My Trading Bot Alerts")
4. **Add a description** (optional)
5. **Make it Public** (recommended for easier setup)
6. **Choose a username** (e.g., "mytradingalerts")
7. **Tap "Create"**

### Step 2: Add Your Bot as Admin

1. **Open your channel**
2. **Tap the channel name** at the top
3. **Tap "Administrators"**
4. **Tap "Add Admin"**
5. **Search for your bot** by username
6. **Add the bot** and give it these permissions:
   - ✅ **Post Messages**
   - ✅ **Edit Messages**
   - ✅ **Delete Messages** (optional)

### Step 3: Get Your Bot Token

1. **Message @BotFather** on Telegram
2. **Send `/newbot`** (if you haven't created a bot yet)
3. **Follow the instructions** to create your bot
4. **Copy the bot token** (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### Step 4: Get Your User ID

1. **Start a chat** with your bot
2. **Send any message** (e.g., "hello")
3. **Run the setup script**:
   ```bash
   python setup_channel.py
   ```

### Step 5: Configure Your Bot

The setup script will automatically:
- ✅ Test your bot token
- ✅ Get your user ID
- ✅ Test channel access
- ✅ Update your `config.toml`

## 📝 Configuration Example

After setup, your `config.toml` will look like:

```toml
# Telegram & Discord
tg_token   = "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"  # Your bot token
tg_chat    = "-1001234567890"                        # Channel ID (negative number)
tg_authorized_users = ["123456789"]                  # Your user ID
discord_webhook = "https://discord.com/api/webhooks/..."
```

## 🔍 Understanding the Setup

### Channel ID vs User ID
- **Channel ID**: Negative number (e.g., `-1001234567890`) - where notifications are sent
- **User ID**: Positive number (e.g., `123456789`) - who can control the bot

### How It Works
1. **Notifications** (trades, profits, errors) → Sent to the channel
2. **Bot Commands** (start, stop, reset) → Sent via private messages to you
3. **You control the bot** → Through private messages with the bot

## 🧪 Testing the Setup

### Test 1: Channel Notifications
```bash
cargo run
```
You should see notifications appear in your channel.

### Test 2: Bot Commands
1. **Send `/start`** to your bot in private chat
2. **Send `/status`** to check bot status
3. **Send `/profit`** to see profit info

## 🚨 Common Issues

### Issue 1: "Bot was blocked by the user"
- **Solution**: Unblock your bot in Telegram settings

### Issue 2: "Chat not found"
- **Solution**: Make sure the bot is added as admin to the channel

### Issue 3: "Forbidden: bot is not a member"
- **Solution**: Add the bot as an admin to the channel

### Issue 4: "Forbidden: bot was blocked"
- **Solution**: Remove and re-add the bot as admin

## 🔄 Alternative: Private Channel

If you want a private channel:

1. **Create a private channel** instead of public
2. **Add your bot as admin**
3. **Get the channel invite link**
4. **Use the channel ID** (will be a negative number)

## 📱 Channel Management

### Adding More Admins
- **Channel admins** can also control the bot
- **Add their user IDs** to `tg_authorized_users` array

### Multiple Channels
- **Currently supports one notification channel**
- **You can modify the code** to support multiple channels

## 🎉 Benefits of Channel Setup

✅ **Public notifications** - Anyone can see trading activity  
✅ **Professional appearance** - Dedicated channel for alerts  
✅ **Easy sharing** - Share channel link with others  
✅ **Separate control** - You control the bot privately  
✅ **Better organization** - Keep trading alerts separate from personal chats  

## 🚀 Next Steps

1. **Run the setup script**: `python setup_channel.py`
2. **Test the configuration**: `cargo run`
3. **Send test commands** to your bot
4. **Monitor the channel** for notifications

## 📞 Need Help?

If you encounter issues:
1. **Check bot permissions** in the channel
2. **Verify channel is public** (or bot is admin)
3. **Test bot token** manually
4. **Check user ID** is correct
5. **Restart the application** after changes 