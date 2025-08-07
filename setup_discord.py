#!/usr/bin/env python3
"""
Discord Configuration Setup Script
"""

import toml
import re

def get_discord_token():
    """Guide user to get Discord token"""
    print("\n🔑 Discord Token Setup")
    print("=" * 30)
    print("You need to get your Discord token. Choose an option:")
    print("\n1. User Token (Recommended for signal scraping)")
    print("   - Open Discord in browser (discord.com)")
    print("   - Press F12 → Network tab")
    print("   - Send a message in any channel")
    print("   - Look for requests to discord.com/api/v9/channels/...")
    print("   - Find 'authorization' header (starts with MTI...)")
    print("\n2. Bot Token")
    print("   - Go to discord.com/developers/applications")
    print("   - Create/select application → Bot section")
    print("   - Create/Reset Token (starts with 'Bot ')")
    
    token = input("\nEnter your Discord token: ").strip()
    
    if not token:
        print("❌ Token cannot be empty")
        return None
    
    if token == "YOUR_DISCORD_USER_TOKEN":
        print("❌ Please replace with your actual Discord token")
        return None
    
    return token

def get_channel_ids():
    """Guide user to get Discord channel IDs"""
    print("\n📺 Discord Channel ID Setup")
    print("=" * 30)
    print("To get channel ID:")
    print("1. Enable Developer Mode in Discord:")
    print("   - User Settings → Advanced → Developer Mode ✅")
    print("2. Right-click on target channel")
    print("3. Select 'Copy ID'")
    print("4. Channel ID looks like: 1234567890123456789")
    
    channels_input = input("\nEnter channel ID(s) separated by commas: ").strip()
    
    if not channels_input:
        print("❌ Channel ID cannot be empty")
        return None
    
    # Split by comma and clean up
    channels = [ch.strip() for ch in channels_input.split(",") if ch.strip()]
    
    # Validate format
    for channel in channels:
        if not channel.isdigit():
            print(f"⚠️  Warning: Channel ID '{channel}' should be numeric")
        elif len(channel) < 17 or len(channel) > 21:
            print(f"⚠️  Warning: Channel ID '{channel}' length seems incorrect")
    
    return channels

def update_config(token, channels):
    """Update config.toml with Discord settings"""
    try:
        # Read current config
        with open("config.toml", "r") as f:
            config = toml.load(f)
        
        # Update Discord settings
        config["discord_token"] = token
        config["discord_channel_id"] = channels
        
        # Write back to file
        with open("config.toml", "w") as f:
            toml.dump(config, f)
        
        print("\n✅ Configuration updated successfully!")
        return True
        
    except Exception as e:
        print(f"❌ Error updating config: {e}")
        return False

def test_config():
    """Test the current configuration"""
    print("\n🧪 Testing Configuration")
    print("=" * 30)
    
    try:
        with open("config.toml", "r") as f:
            config = toml.load(f)
        
        token = config.get("discord_token", "")
        channels = config.get("discord_channel_id", [])
        
        print(f"Token: {'✅ Set' if token and token != 'YOUR_DISCORD_USER_TOKEN' else '❌ Not set'}")
        print(f"Channels: {'✅ Set' if channels and channels != ['DISCORD_CHANNEL_ID'] else '❌ Not set'}")
        
        if token and token != "YOUR_DISCORD_USER_TOKEN":
            print(f"Token type: {'Bot' if token.startswith('Bot ') else 'User'}")
            print(f"Token preview: {token[:10]}...")
        
        if channels and channels != ["DISCORD_CHANNEL_ID"]:
            print(f"Target channels: {channels}")
        
        return True
        
    except Exception as e:
        print(f"❌ Error reading config: {e}")
        return False

def main():
    print("🔧 Discord Configuration Setup")
    print("=" * 40)
    
    # Test current config first
    test_config()
    
    # Ask if user wants to update
    update = input("\nDo you want to update Discord configuration? (y/n): ").lower().strip()
    
    if update != 'y':
        print("Configuration not updated.")
        return
    
    # Get Discord token
    token = get_discord_token()
    if not token:
        return
    
    # Get channel IDs
    channels = get_channel_ids()
    if not channels:
        return
    
    # Update configuration
    if update_config(token, channels):
        print("\n🎉 Setup complete! You can now:")
        print("1. Run: python verify_config.py")
        print("2. Run: python test_discord_parsing.py")
        print("3. Run: RUST_LOG=debug cargo run")
        
        # Test the new config
        print("\n🔍 Testing new configuration...")
        test_config()

if __name__ == "__main__":
    main() 