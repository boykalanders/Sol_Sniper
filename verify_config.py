#!/usr/bin/env python3
"""
Simple script to verify Discord configuration
"""
import toml
import sys

def verify_discord_config():
    try:
        # Read config.toml
        with open('config.toml', 'r') as f:
            config = toml.load(f)
        
        print("🔍 Verifying Discord configuration...")
        
        # Check token
        token = config.get('discord_token', '')
        if not token:
            print("❌ No discord_token found in config.toml")
            return False
        
        # Check if it's a bot token
        is_bot = token.startswith('Bot ')
        print(f"📋 Token type: {'Bot' if is_bot else 'User'}")
        
        if is_bot:
            print(f"🤖 Bot token length: {len(token.replace('Bot ', ''))}")
        else:
            print(f"👤 User token length: {len(token)}")
        
        # Check channel IDs
        channels = config.get('discord_channel_id', [])
        if not channels:
            print("❌ No discord_channel_id found in config.toml")
            return False
        
        print(f"📺 Target channels ({len(channels)}):")
        for i, channel in enumerate(channels):
            channel_str = str(channel)
            print(f"  {i+1}. {channel_str} (length: {len(channel_str)})")
            
            # Validate channel ID format
            if not channel_str.isdigit():
                print(f"    ⚠️  Warning: Channel ID should be numeric")
            elif len(channel_str) < 17 or len(channel_str) > 21:
                print(f"    ⚠️  Warning: Channel ID length seems incorrect")
            else:
                print(f"    ✅ Valid format")
        
        print("\n📝 Configuration Summary:")
        print(f"  Token: {'✅ Present' if token else '❌ Missing'}")
        print(f"  Type: {'Bot' if is_bot else 'User'}")
        print(f"  Channels: {len(channels)} configured")
        
        return True
        
    except FileNotFoundError:
        print("❌ config.toml not found")
        return False
    except Exception as e:
        print(f"❌ Error reading config: {e}")
        return False

if __name__ == "__main__":
    if verify_discord_config():
        print("\n✅ Config verification completed")
    else:
        print("\n❌ Config verification failed")
        sys.exit(1)