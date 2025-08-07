#!/usr/bin/env python3
"""
Test script to debug Discord message parsing
"""

import re
import toml

def test_message_parsing():
    """Test the message parsing logic"""
    
    # Test messages
    test_messages = [
        "CA: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53",
        "CA=i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53",
        "CA i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53",
        "Contract Address: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53",
        "Token: i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53",
        "i9iE6yj9pEtkC3mHUpA16kKvHvhRkBobAsAKUm5vr53",
        "CA: invalid_address_here",
        "No CA here",
    ]
    
    print("🧪 Testing Discord Message Parsing")
    print("=" * 40)
    
    for i, message in enumerate(test_messages, 1):
        print(f"\n📝 Test {i}: '{message}'")
        
        # Check for CA signal
        signal_pattern = r"(?i)\b(CA)\b"
        has_signal = bool(re.search(signal_pattern, message))
        
        if has_signal:
            print("✅ CA signal detected")
            
            # Test token patterns
            token_patterns = [
                r"(?i)ca\s*:\s*([A-Za-z0-9]{32,44})",
                r"(?i)ca\s*=\s*([A-Za-z0-9]{32,44})",
                r"(?i)ca\s+([A-Za-z0-9]{32,44})",
                r"([A-Za-z0-9]{32,44})",
            ]
            
            for j, pattern in enumerate(token_patterns, 1):
                matches = re.findall(pattern, message)
                if matches:
                    print(f"✅ Pattern {j} matched: {matches}")
                    for match in matches:
                        if len(match) >= 32 and len(match) <= 44:
                            print(f"✅ Valid length address: {match}")
                        else:
                            print(f"❌ Invalid length: {match} (length: {len(match)})")
                    break
            else:
                print("❌ No token pattern matched")
        else:
            print("❌ No CA signal detected")

def check_config():
    """Check the current configuration"""
    try:
        with open("config.toml", "r") as f:
            config = toml.load(f)
        
        print("\n📋 Configuration Check")
        print("=" * 30)
        
        # Check Discord token
        discord_token = config.get("discord_token", "")
        if discord_token and discord_token != "YOUR_DISCORD_USER_TOKEN":
            print(f"✅ Discord token: {discord_token[:10]}...")
        else:
            print("❌ Discord token: Using placeholder value")
        
        # Check Discord channel IDs
        channel_ids = config.get("discord_channel_id", [])
        if channel_ids and channel_ids != ["DISCORD_CHANNEL_ID"]:
            print(f"✅ Discord channels: {channel_ids}")
        else:
            print("❌ Discord channels: Using placeholder values")
        
        # Check Telegram config
        tg_token = config.get("tg_token", "")
        if tg_token and tg_token != "YOUR_TELEGRAM_BOT_TOKEN":
            print(f"✅ Telegram token: {tg_token[:10]}...")
        else:
            print("❌ Telegram token: Using placeholder value")
            
    except Exception as e:
        print(f"❌ Error reading config: {e}")

if __name__ == "__main__":
    test_message_parsing()
    check_config()
    
    print("\n🔧 Troubleshooting Tips:")
    print("1. Make sure discord_channel_id in config.toml is set to real channel IDs")
    print("2. Ensure your Discord token is valid and has proper permissions")
    print("3. Check that the bot is in the target channels")
    print("4. Verify the message format matches the expected patterns") 