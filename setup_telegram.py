#!/usr/bin/env python3
"""
Comprehensive Telegram Bot Setup Script
"""

import requests
import json
import re
import os

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
    # Replace the field value
    pattern = rf'({field}\s*=\s*)"[^"]*"'
    replacement = rf'\1"{value}"'
    updated = re.sub(pattern, replacement, content)
    
    if updated == content:
        # Field not found, add it
        lines = content.split('\n')
        for i, line in enumerate(lines):
            if line.strip().startswith('# Telegram & Discord'):
                # Insert after the comment
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

def main():
    print("🤖 Telegram Bot Setup Wizard")
    print("=" * 40)
    
    # Read current config
    config_content = read_config()
    if not config_content:
        print("❌ Could not read config.toml")
        return
    
    # Check current bot token
    current_token_match = re.search(r'tg_token\s*=\s*"([^"]+)"', config_content)
    current_token = current_token_match.group(1) if current_token_match else ""
    
    if current_token and current_token != "YOUR_TELEGRAM_BOT_TOKEN":
        print(f"✅ Found existing bot token: {current_token[:10]}...")
        
        # Test the token
        is_valid, bot_info = test_bot_token(current_token)
        if is_valid:
            print(f"✅ Bot token is valid!")
            print(f"   Bot name: {bot_info.get('first_name', 'Unknown')}")
            print(f"   Bot username: @{bot_info.get('username', 'Unknown')}")
            
            # Check for user ID
            user_id, username, name = get_user_id(current_token)
            if user_id:
                print(f"✅ Found user: {name} (@{username}) - ID: {user_id}")
                
                # Update config with user ID
                updated_config = update_config_field(config_content, "tg_chat", str(user_id))
                updated_config = update_config_field(updated_config, "tg_authorized_users", f'["{user_id}"]')
                
                if write_config(updated_config):
                    print("✅ Updated config.toml with user ID!")
                    print("🎉 Setup complete! You can now run: cargo run")
                else:
                    print("❌ Failed to update config.toml")
            else:
                print("❌ No messages found from user")
                print("📱 Please send a message to your bot first, then run this script again")
        else:
            print("❌ Bot token is invalid")
            print("🔧 Please get a new token from @BotFather")
    else:
        print("❌ No valid bot token found")
        print("\n🔧 To get a bot token:")
        print("   1. Open Telegram and message @BotFather")
        print("   2. Send /newbot and follow instructions")
        print("   3. Copy the token (looks like: 123456789:ABCdefGHIjklMNOpqrsTUVwxyz)")
        print("   4. Update your config.toml with: tg_token = \"YOUR_TOKEN_HERE\"")
        print("   5. Run this script again")

if __name__ == "__main__":
    main() 