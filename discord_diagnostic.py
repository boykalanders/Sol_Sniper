#!/usr/bin/env python3
"""
Discord Connection Diagnostic Script
"""

import asyncio
import json
import websockets
import toml
import sys

async def test_discord_connection():
    """Test Discord WebSocket connection"""
    print("🔍 Discord Connection Diagnostic")
    print("=" * 40)
    
    # Load config
    try:
        with open("config.toml", "r") as f:
            config = toml.load(f)
    except Exception as e:
        print(f"❌ Error reading config.toml: {e}")
        return
    
    token = config.get("discord_token", "")
    channels = config.get("discord_channel_id", [])
    
    print(f"Token: {'✅ Set' if token and token != 'YOUR_DISCORD_USER_TOKEN' else '❌ Not set'}")
    print(f"Channels: {channels}")
    
    if not token or token == "YOUR_DISCORD_USER_TOKEN":
        print("\n❌ Please set your Discord token first!")
        print("Run: python setup_discord.py")
        return
    
    if not channels or channels == ["DISCORD_CHANNEL_ID"]:
        print("\n❌ Please set your Discord channel IDs first!")
        print("Run: python setup_discord.py")
        return
    
    print(f"\n🔗 Connecting to Discord Gateway...")
    
    try:
        # Connect to Discord Gateway
        uri = "wss://gateway.discord.gg/?v=10&encoding=json"
        websocket = await websockets.connect(uri)
        
        print("✅ WebSocket connected")
        
        # Send identify payload
        is_bot_token = token.startswith("Bot ")
        
        identify = {
            "op": 2,
            "d": {
                "token": token,
                "properties": {
                    "$os": "Windows",
                    "$browser": "Chrome",
                    "$device": "Desktop"
                }
            }
        }
        
        if is_bot_token:
            identify["d"]["intents"] = 33280
        
        print(f"📤 Sending identify payload (token type: {'Bot' if is_bot_token else 'User'})")
        await websocket.send(json.dumps(identify))
        
        # Listen for responses
        print("👂 Listening for Discord responses...")
        print("(Press Ctrl+C to stop)")
        
        while True:
            try:
                message = await asyncio.wait_for(websocket.recv(), timeout=30.0)
                data = json.loads(message)
                
                op_code = data.get("op")
                event_type = data.get("t")
                
                print(f"\n📨 Received: op={op_code}, type={event_type}")
                
                if op_code == 10:  # Hello
                    heartbeat_interval = data["d"]["heartbeat_interval"]
                    print(f"✅ HELLO received, heartbeat interval: {heartbeat_interval}ms")
                    
                elif op_code == 0 and event_type == "READY":
                    user = data["d"]["user"]
                    username = user.get("username", "Unknown")
                    user_id = user.get("id", "Unknown")
                    print(f"✅ READY received - Logged in as: {username} ({user_id})")
                    print(f"🎯 Target channels: {channels}")
                    
                elif op_code == 0 and event_type == "MESSAGE_CREATE":
                    message_data = data["d"]
                    channel_id = message_data.get("channel_id", "")
                    author = message_data.get("author", {})
                    author_name = author.get("username", "Unknown")
                    content = message_data.get("content", "")
                    is_bot = author.get("bot", False)
                    
                    print(f"📨 Message from {author_name} in {channel_id}: {content}")
                    
                    if channel_id in channels:
                        print(f"🎯 Message is from target channel!")
                        if is_bot:
                            print("🤖 Skipping bot message")
                        else:
                            print("✅ Processing user message")
                    else:
                        print(f"❌ Message not from target channel (expected: {channels})")
                        
                elif op_code == 4:  # Authentication failed
                    error_code = data.get("d", 0)
                    print(f"❌ Authentication failed with code: {error_code}")
                    if error_code == 4004:
                        print("   Invalid token")
                    elif error_code == 4011:
                        print("   Disallowed intents")
                    elif error_code == 4013:
                        print("   Invalid intents")
                    elif error_code == 4014:
                        print("   Disallowed intents (privileged)")
                    break
                    
                elif op_code == 9:  # Invalid session
                    print("❌ Invalid session - token may be expired")
                    break
                    
                elif op_code == 7:  # Reconnect
                    print("🔄 Reconnect requested")
                    break
                    
            except asyncio.TimeoutError:
                print("⏰ No message received in 30 seconds")
                break
                
    except websockets.exceptions.ConnectionClosed as e:
        print(f"❌ WebSocket connection closed: {e}")
    except Exception as e:
        print(f"❌ Connection error: {e}")
    finally:
        if 'websocket' in locals():
            await websocket.close()

def main():
    try:
        asyncio.run(test_discord_connection())
    except KeyboardInterrupt:
        print("\n👋 Diagnostic stopped by user")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    main() 