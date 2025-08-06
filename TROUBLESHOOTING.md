# Troubleshooting Guide

## Common Issues and Solutions

### 1. Insufficient SOL Balance

**Problem**: 
```
💰 Current SOL Balance: 0.0000 SOL
📊 Trades possible with current balance: 0
⚠️ WARNING: Balance (0.0000 SOL) is less than trade amount (0.2 SOL)
```

**Solution**:
1. **Fund your wallet**: Send SOL to your wallet address (shown in logs)
2. **Reduce trade amount**: Edit `config.toml` and reduce `amount_sol` value
3. **Check wallet**: Ensure you're using the correct keypair file

**Quick Fix**:
```bash
# Fund your wallet with at least 0.2 SOL
# Or reduce amount_sol in config.toml to a smaller value like 0.1
```

### 2. Telegram Bot "TerminatedByOtherGetUpdates" Error

**Problem**:
```
ERROR teloxide::error_handlers: An error from the update listener: Api(TerminatedByOtherGetUpdates)
```

**Cause**: Multiple bot instances are running with the same Telegram token

**Solution**:
1. **Kill existing instances**:
   ```bash
   pkill -f snipe
   pkill -f telegram
   ```

2. **Wait 30 seconds** for Telegram to release the token

3. **Restart your bot**:
   ```bash
   cargo run
   ```

4. **Use the check script**:
   ```bash
   ./scripts/check_bot_instances.sh
   ```

### 3. Discord Connection Issues

**Problem**: Discord bot fails to connect or authenticate

**Solutions**:
1. **Check token validity**: Ensure your Discord token is correct and not expired
2. **Verify permissions**: Make sure the bot/user has access to the target channels
3. **Check network**: Ensure stable internet connection

### 4. RPC Connection Issues

**Problem**: Cannot connect to Solana RPC

**Solutions**:
1. **Check RPC URL**: Verify the RPC endpoint in `config.toml`
2. **Try different RPC**: Use a different RPC provider if one is down
3. **Check rate limits**: Some free RPCs have rate limits

### 5. Keypair File Issues

**Problem**: Cannot read keypair file

**Solutions**:
1. **Check file path**: Ensure `keys/id.json` exists
2. **Verify format**: Keypair should be in JSON format
3. **Check permissions**: Ensure the file is readable

## Prevention Tips

1. **Always check balance before starting**: Ensure you have sufficient SOL
2. **Use unique Telegram tokens**: Don't share bot tokens between instances
3. **Monitor logs**: Watch for warnings and errors
4. **Backup configuration**: Keep a backup of your working config
5. **Test with small amounts**: Start with small trade amounts to test

## Getting Help

If you're still having issues:

1. **Check the logs**: Look for specific error messages
2. **Verify configuration**: Ensure all settings in `config.toml` are correct
3. **Test components**: Try each component (Discord, Telegram, RPC) separately
4. **Check dependencies**: Ensure all required software is installed and up to date

## Emergency Commands

```bash
# Kill all bot instances
pkill -f snipe

# Check running processes
ps aux | grep snipe

# Check wallet balance (replace with your RPC)
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getBalance","params":["FMTqEscgoB5oFSRktWKhv2VzoeJEMjkn4zdJ9PRFhrLk"]}' https://api.mainnet-beta.solana.com

# Reset Telegram webhook (if needed)
curl -X POST "https://api.telegram.org/bot<YOUR_BOT_TOKEN>/deleteWebhook"
``` 