#!/usr/bin/env python3
"""
Telegram Channel Setup Script
Helps you set up bot notifications to a Telegram channel
"""

import requests
import json
import re

def read_config():
    """Read current config.toml"""
    try:
        with open("config.toml", "r") as f:
            content = f.read()
            return content
    except Exception as e:
        print(f"❌ Error reading config.toml: {e}")
        return ""

def write_config(content):
    """Write updated config.toml"""
    try:
        with open("config.toml", "w") as f:
            f.write(content)
        return True
    except Exception as e:
        print(f"❌ Error writing config.toml: {e}")
        return False

def update_config_field(content, field, value):
    """Update a specific field in config.toml"""
    pattern = rf'({field}\s*=\s*)"[^"]*"'
    replacement = rf'\1"{value}"'
    updated = re.sub(pattern, replacement, content)
    
    if updated == content:
        # Field not found, add it
        lines = content.split('\n')
        for i, line in enumerate(lines):
            if line.strip().startswith('# Telegram & Discord'):
                lines.insert(i + 1, f'{field} = "{value}"')
                break
        updated = '\n'.join(lines)
    
    return updated

def test_bot_token(token):
    """Test if bot token is valid"""
    url = f"https://api.telegram.org/bot{token}/getMe"
    try:
        response = requests.get(url)
        data = response.json()
        return data.get("ok", False), data.get("result", {})
    except:
        return False, {}

def get_user_id(token):
    """Get user ID from bot updates"""
    url = f"https://api.telegram.org/bot{token}/getUpdates"
    try:
        response = requests.get(url)
        data = response.json()
        
        if data.get("ok"):
            updates = data.get("result", [])
            if updates:
                latest = updates[-1]
                if "message" in latest:
                    user = latest["message"]["from"]
                    return user["id"], user.get("username", ""), user.get("first_name", "")
        return None, "", ""
    except:
        return None, "", ""

def get_channel_id(token, channel_username):
    """Get channel ID by sending a test message"""
    # Remove @ if present
    if channel_username.startswith('@'):
        channel_username = channel_username[1:]
    
    # Try to send a test message to get channel info
    url = f"https://api.telegram.org/bot{token}/sendMessage"
    test_message = "🤖 Bot test message - this will be deleted"
    
    try:
        response = requests.post(url, json={
            "chat_id": f"@{channel_username}",
            "text": test_message
        })
        data = response.json()
        
        if data.get("ok"):
            # Get the message ID to delete it
            message_id = data["result"]["message_id"]
            chat_id = data["result"]["chat"]["id"]
            
            # Delete the test message
            delete_url = f"https://api.telegram.org/bot{token}/deleteMessage"
            requests.post(delete_url, json={
                "chat_id": chat_id,
                "message_id": message_id
            })
            
            return chat_id, channel_username
        else:
            print(f"❌ Error: {data.get('description', 'Unknown error')}")
            return None, None
    except Exception as e:
        print(f"❌ Error: {e}")
        return None, None

def main():
    print("📢 Telegram Channel Setup")
    print("=" * 30)
    
    # Read current config
    config_content = read_config()
    if not config_content:
        print("❌ Could not read config.toml")
        return
    
    # Check current bot token
    current_token_match = re.search(r'tg_token\s*=\s*"([^"]+)"', config_content)
    current_token = current_token_match.group(1) if current_token_match else ""
    
    if not current_token or current_token == "YOUR_TELEGRAM_BOT_TOKEN":
        print("❌ No valid bot token found")
        print("📝 Please update your config.toml with a real bot token first")
        return
    
    # Test the token
    is_valid, bot_info = test_bot_token(current_token)
    if not is_valid:
        print("❌ Bot token is invalid")
        return
    
    print(f"✅ Bot: {bot_info.get('first_name', 'Unknown')} (@{bot_info.get('username', 'Unknown')})")
    
    # Get user ID for authorized users
    user_id, username, name = get_user_id(current_token)
    if not user_id:
        print("❌ No messages found from user")
        print("📱 Please send a message to your bot first")
        return
    
    print(f"✅ User: {name} (@{username}) - ID: {user_id}")
    
    # Get channel information
    print("\n📢 Channel Setup:")
    print("1. Create a new Telegram channel (or use existing)")
    print("2. Add your bot as an admin to the channel")
    print("3. Give the bot permission to send messages")
    print("4. Copy the channel username (e.g., 'mytradingchannel')")
    
    channel_username = input("\nEnter channel username (without @): ").strip()
    if not channel_username:
        print("❌ No channel username provided")
        return
    
    # Test channel access
    print(f"\n🔍 Testing channel access...")
    channel_id, _ = get_channel_id(current_token, channel_username)
    
    if channel_id:
        print(f"✅ Channel access confirmed!")
        print(f"   Channel ID: {channel_id}")
        print(f"   Channel: @{channel_username}")
        
        # Update config
        updated_config = update_config_field(config_content, "tg_chat", str(channel_id))
        updated_config = update_config_field(updated_config, "tg_authorized_users", f'["{user_id}"]')
        
        if write_config(updated_config):
            print("\n✅ Updated config.toml!")
            print("📝 Configuration:")
            print(f"   tg_token = \"{current_token}\"")
            print(f"   tg_chat = \"{channel_id}\"  # Channel for notifications")
            print(f"   tg_authorized_users = [\"{user_id}\"]  # You can control the bot")
            
            print("\n🎉 Setup complete!")
            print("📢 Notifications will be sent to the channel")
            print("🤖 You can control the bot via private messages")
            print("🚀 Run: cargo run")
        else:
            print("❌ Failed to update config.toml")
    else:
        print("❌ Could not access channel")
        print("🔧 Make sure:")
        print("   1. Channel exists and is public")
        print("   2. Bot is added as admin")
        print("   3. Bot has permission to send messages")

if __name__ == "__main__":
    main() 