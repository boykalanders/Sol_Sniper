# Telegram Bot Control Panel

This module adds Telegram bot functionality to control your sniper bot remotely. You can reset profit data, stop/start the bot, and monitor its status through Telegram commands.

## Features

- **🔐 Secure Authentication**: Only authorized users can control the bot
- **💰 Profit Management**: Reset profit data remotely
- **🛑 Bot Control**: Stop and start the bot remotely
- **📊 Status Monitoring**: Get real-time bot status and profit statistics
- **🔔 Notifications**: Receive notifications about bot activities

## Setup Instructions

### 1. Create a Telegram Bot

1. **Start a chat with @BotFather** on Telegram
2. **Send `/newbot`** and follow the instructions
3. **Choose a name** for your bot (e.g., "Sniper Bot Controller")
4. **Choose a username** (must end with 'bot', e.g., "my_sniper_bot")
5. **Save the bot token** that BotFather gives you

### 2. Get Your User ID

You need to add your Telegram user ID to the authorized users list. Run the helper script:

```bash
# Set your bot token as environment variable
export TELEGRAM_BOT_TOKEN="YOUR_BOT_TOKEN_HERE"

# Run the helper script
cargo run --bin get_telegram_id
```

Then:
1. **Start a chat** with your bot on Telegram
2. **Send any message** to the bot
3. **Copy your User ID** from the response
4. **Stop the helper script** with Ctrl+C

### 3. Update Configuration

Edit your `config.toml` file:

```toml
# Telegram Bot Configuration
tg_token = "YOUR_BOT_TOKEN_HERE"
tg_chat = "@your_channel_or_group"  # Optional: for notifications
tg_authorized_users = ["YOUR_USER_ID_HERE", "ANOTHER_USER_ID"]
```

### 4. Start the Bot

The Telegram bot will automatically start with your main sniper bot:

```bash
cargo run
```

## Available Commands

Once set up, you can send these commands to your Telegram bot:

### 📊 Profit Management

- **`/profit`** - Show current profit statistics
  ```
  📊 Profit Statistics
  
  💰 Profit Summary:
  Total Profit: 2.5000 SOL
  Total Trades: 15
  Winning Trades: 12
  Losing Trades: 3
  Win Rate: 80.0%
  Largest Win: 0.8000 SOL
  Largest Loss: -0.2000 SOL
  Last Updated: 2024-01-15 14:30:25
  ```

- **`/reset`** - Reset all profit data to zero
  ```
  ✅ Profit data reset successfully!
  
  All profit statistics have been reset to zero.
  ```

### 🛑 Bot Control

- **`/stop`** - Stop the bot (pause trading)
  ```
  🛑 Bot stopped successfully!
  
  The bot has been paused and will not execute new trades.
  Use /start_bot to resume trading.
  ```

- **`/start_bot`** - Start the bot (resume trading)
  ```
  ✅ Bot started successfully!
  
  The bot is now active and will execute trades based on signals.
  ```

### 📈 Status Monitoring

- **`/status`** - Show bot status and statistics
  ```
  📊 Bot Status
  
  Status: 🟢 ACTIVE
  Total Profit: 2.5000 SOL
  Total Trades: 15
  Win Rate: 80.0%
  Last Updated: 2024-01-15 14:30:25
  ```

- **`/help`** - Show available commands
  ```
  🤖 Sniper Bot Control Panel
  
  Available Commands:
  • /profit - Show current profit statistics
  • /reset - Reset all profit data to zero
  • /stop - Stop the bot (pause trading)
  • /start_bot - Start the bot (resume trading)
  • /status - Show bot status
  • /help - Show this help message
  
  Usage:
  Send any of these commands to control the bot.
  Only authorized users can use these commands.
  ```

## Security Features

### 🔐 Authorization System

- Only users listed in `tg_authorized_users` can control the bot
- Unauthorized users receive an access denied message
- User IDs are validated on every command

### 🛡️ Safe Operations

- All commands are logged for audit purposes
- Critical operations (reset, stop) provide clear confirmations
- Error handling prevents bot crashes from invalid commands

## Integration with Main Bot

The Telegram bot integrates seamlessly with your existing sniper bot:

### 🔄 Bot State Management

- The bot can be stopped/started remotely without restarting the application
- Trading operations check the bot state before executing
- Status is maintained across bot restarts

### 📊 Profit Database Integration

- Direct access to the SQLite profit database
- Real-time profit statistics
- Secure profit reset functionality

### 🔔 Notification System

- Can send notifications to Telegram channels/groups
- Integrates with existing notification system
- Supports both individual chats and group notifications

## Advanced Configuration

### Multiple Authorized Users

You can authorize multiple users by adding their IDs to the array:

```toml
tg_authorized_users = [
    "123456789",  # Your user ID
    "987654321",  # Another user ID
    "555666777"   # Third user ID
]
```

### Channel Notifications

To receive notifications in a Telegram channel or group:

```toml
tg_chat = "@your_channel_name"  # For public channels
# OR
tg_chat = "-1001234567890"      # For private groups (get ID from @userinfobot)
```

## Troubleshooting

### Bot Not Responding

1. **Check bot token** - Ensure the token in `config.toml` is correct
2. **Verify user authorization** - Make sure your user ID is in the authorized list
3. **Check bot permissions** - Ensure the bot can send messages in the chat
4. **Review logs** - Check application logs for error messages

### Authorization Issues

1. **Get correct user ID** - Use the helper script to get your exact user ID
2. **Check format** - User IDs should be strings in the config array
3. **Restart bot** - Changes to authorized users require a restart

### Command Not Working

1. **Check command syntax** - Commands are case-insensitive but must be exact
2. **Verify bot state** - Some commands may not work if bot is in certain states
3. **Check database** - Ensure the profit database is accessible

## Example Usage Scenarios

### Daily Profit Check
```
User: /profit
Bot: [Shows current profit statistics]

User: /status  
Bot: [Shows bot status and summary]
```

### Emergency Stop
```
User: /stop
Bot: [Confirms bot stopped]

User: /start_bot
Bot: [Confirms bot started]
```

### Reset After Session
```
User: /reset
Bot: [Confirms profit data reset]

User: /profit
Bot: [Shows zeroed statistics]
```

## Development

### Adding New Commands

To add new commands, modify the `handle_message` function in `src/telegram_bot.rs`:

```rust
match text.to_lowercase().trim() {
    "/new_command" => {
        self.handle_new_command(msg.chat.id).await;
    }
    // ... existing commands
}
```

### Custom Notifications

Use the `send_telegram_notification` function to send custom messages:

```rust
telegram_bot::send_telegram_notification(
    &config.tg_token,
    &config.tg_chat,
    "Custom notification message"
).await?;
```

## Security Best Practices

1. **Keep bot token secret** - Never share your bot token publicly
2. **Limit authorized users** - Only add trusted users to the authorized list
3. **Monitor usage** - Check logs regularly for unauthorized access attempts
4. **Use strong tokens** - Generate new bot tokens if compromised
5. **Backup configuration** - Keep secure backups of your config files 