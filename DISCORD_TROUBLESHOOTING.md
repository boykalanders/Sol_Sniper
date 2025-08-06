# 🔧 Discord Message Parsing Troubleshooting

## 🚨 Issue: Messages not being detected

Your message format: `CA: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53`

## ✅ Fixed Issues

### 1. **Improved Regex Patterns**
- ✅ Added specific pattern for `CA: <address>` format
- ✅ Added fallback patterns for other formats
- ✅ Added detailed logging for debugging

### 2. **Enhanced Logging**
- ✅ Added info-level logging for all messages
- ✅ Added pattern matching debug info
- ✅ Added validation step logging

## 🔍 Debug Steps

### Step 1: Test Message Parsing
```bash
python test_discord_parsing.py
```

### Step 2: Check Configuration
Make sure your `config.toml` has real values:
```toml
discord_token = "YOUR_REAL_DISCORD_TOKEN"
discord_channel_id = ["REAL_CHANNEL_ID"]
```

### Step 3: Run with Debug Logging
```bash
RUST_LOG=info cargo run
```

## 🚨 Common Issues

### Issue 1: Placeholder Channel ID
**Problem**: `discord_channel_id = ["DISCORD_CHANNEL_ID"]`
**Solution**: Replace with real Discord channel ID

### Issue 2: Invalid Discord Token
**Problem**: Token expired or invalid
**Solution**: Get new token from Discord

### Issue 3: Bot Not in Channel
**Problem**: Bot can't see channel messages
**Solution**: Add bot to target channels

### Issue 4: Message Format
**Problem**: Message doesn't match expected format
**Solution**: Ensure message contains "CA" and valid address

## 🧪 Test Your Setup

1. **Run test script**: `python test_discord_parsing.py`
2. **Check logs**: Look for "📨 Message from target channel"
3. **Verify parsing**: Look for "🔍 Signal detected" messages
4. **Check validation**: Look for "✅ Token address validated"

## 📞 Need Help?

If still not working:
1. Check Discord token permissions
2. Verify channel ID is correct
3. Ensure bot is in target channels
4. Test with simple message format 