#!/usr/bin/env python3
"""
Simple script to get your Telegram user ID
"""

import requests
import json

def get_user_id(bot_token):
    """Get user ID from bot updates"""
    url = f"https://api.telegram.org/bot{bot_token}/getUpdates"
    
    try:
        response = requests.get(url)
        data = response.json()
        
        if data.get("ok"):
            updates = data.get("result", [])
            if updates:
                # Get the latest message
                latest = updates[-1]
                if "message" in latest:
                    user = latest["message"]["from"]
                    user_id = user["id"]
                    username = user.get("username", "No username")
                    first_name = user.get("first_name", "No name")
                    
                    print(f"✅ Found your information:")
                    print(f"   User ID: {user_id}")
                    print(f"   Username: @{username}")
                    print(f"   Name: {first_name}")
                    print(f"\n📝 Add this to your config.toml:")
                    print(f"   tg_authorized_users = [\"{user_id}\"]")
                    print(f"   tg_chat = \"{user_id}\"")
                    return user_id
            else:
                print("❌ No messages found. Please send a message to your bot first.")
                print("   Then run this script again.")
        else:
            print(f"❌ Error: {data.get('description', 'Unknown error')}")
            
    except Exception as e:
        print(f"❌ Error: {e}")
    
    return None

def test_bot_token(bot_token):
    """Test if bot token is valid"""
    url = f"https://api.telegram.org/bot{bot_token}/getMe"
    
    try:
        response = requests.get(url)
        data = response.json()
        
        if data.get("ok"):
            bot_info = data["result"]
            print(f"✅ Bot token is valid!")
            print(f"   Bot name: {bot_info['first_name']}")
            print(f"   Bot username: @{bot_info['username']}")
            return True
        else:
            print(f"❌ Invalid bot token: {data.get('description', 'Unknown error')}")
            return False
            
    except Exception as e:
        print(f"❌ Error testing token: {e}")
        return False

def read_config():
    """Read bot token from config.toml"""
    try:
        import toml
        with open("config.toml", "r") as f:
            config = toml.load(f)
            return config.get("tg_token", "")
    except Exception as e:
        print(f"❌ Error reading config.toml: {e}")
        return ""

if __name__ == "__main__":
    print("🤖 Telegram User ID Helper")
    print("=" * 30)
    
    # Try to read from config.toml first
    bot_token = read_config()
    
    if not bot_token or bot_token == "YOUR_TELEGRAM_BOT_TOKEN":
        print("❌ No valid bot token found in config.toml")
        print("📝 Please update your config.toml with a real bot token first")
        print("   Example: tg_token = \"123456789:ABCdefGHIjklMNOpqrsTUVwxyz\"")
        exit(1)
    
    print(f"\n🔍 Testing bot token...")
    if test_bot_token(bot_token):
        print(f"\n🔍 Getting user ID...")
        print("   (Make sure you've sent a message to your bot)")
        user_id = get_user_id(bot_token)
        
        if user_id:
            print(f"\n🎉 Success! Update your config.toml with:")
            print(f"   tg_token = \"{bot_token}\"")
            print(f"   tg_chat = \"{user_id}\"")
            print(f"   tg_authorized_users = [\"{user_id}\"]")
    else:
        print("❌ Please check your bot token and try again.") 