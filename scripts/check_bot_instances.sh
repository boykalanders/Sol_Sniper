#!/bin/bash

# Script to check for multiple bot instances and help resolve Telegram bot conflicts

echo "🔍 Checking for running bot instances..."

# Check for Rust processes that might be our bot
echo "📊 Rust processes:"
ps aux | grep -E "(snipe|cargo|rust)" | grep -v grep

echo ""
echo "🤖 Telegram bot processes:"
ps aux | grep -E "(telegram|bot)" | grep -v grep

echo ""
echo "💡 If you see multiple instances, you can kill them with:"
echo "   pkill -f snipe"
echo "   pkill -f telegram"
echo ""
echo "🔧 To resolve Telegram 'TerminatedByOtherGetUpdates' error:"
echo "   1. Kill all existing bot instances"
echo "   2. Wait 30 seconds for Telegram to release the token"
echo "   3. Restart your bot"
echo ""
echo "💰 To fund your wallet, send SOL to the address shown in the logs"
echo "   Current wallet: FMTqEscgoB5oFSRktWKhv2VzoeJEMjkn4zdJ9PRFhrLk" 