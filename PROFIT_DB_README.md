# Profit Database Module

This module provides SQLite-based profit tracking functionality for the sniper bot. It allows you to store, retrieve, and reset profit data from trading activities.

## Features

- **Persistent Storage**: All profit data is stored in a SQLite database (`profit_tracking.db`)
- **Comprehensive Tracking**: Tracks total profit, number of trades, win/loss counts, largest win/loss
- **Easy Integration**: Simple API for getting and resetting profit data
- **Thread-Safe**: Can be safely used across multiple threads

## Database Schema

The profit tracking database contains a single table with the following structure:

```sql
CREATE TABLE profit_tracking (
    id INTEGER PRIMARY KEY,
    total_profit REAL NOT NULL DEFAULT 0.0,
    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,
    largest_win REAL NOT NULL DEFAULT 0.0,
    largest_loss REAL NOT NULL DEFAULT 0.0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Usage

### 1. Initialize the Database

```rust
use snipe::profit_db::ProfitDatabase;

// Initialize the profit database
let profit_db = ProfitDatabase::new("profit_tracking.db")?;
```

### 2. Get Profit Data

#### Get Complete Profit Statistics
```rust
// Get detailed profit statistics
let stats = profit_db.get_profit()?;
println!("Total Profit: {:.4} SOL", stats.total_profit);
println!("Total Trades: {}", stats.total_trades);
println!("Win Rate: {:.1}%", stats.win_rate());
```

#### Get Simple Profit Value
```rust
// Get just the profit value as f64
let profit = profit_db.get_profit_value()?;
println!("Current Profit: {:.4} SOL", profit);
```

#### Get Formatted Summary
```rust
// Get a formatted string summary
let summary = profit_db.get_profit_summary()?;
println!("{}", summary);
```

### 3. Add Profit from Trades

```rust
// Add profit from a winning trade
profit_db.add_profit(0.5)?;  // +0.5 SOL profit

// Add loss from a losing trade
profit_db.add_profit(-0.2)?; // -0.2 SOL loss
```

### 4. Reset Profit Data

```rust
// Reset all profit data to zero
profit_db.reset_profit()?;
```

## API Reference

### ProfitDatabase

#### Methods

- `new(db_path: &str) -> SqliteResult<Self>`
  - Initialize a new profit database connection
  
- `get_profit() -> SqliteResult<ProfitStats>`
  - Get complete profit statistics
  
- `get_profit_value() -> SqliteResult<f64>`
  - Get just the total profit value
  
- `add_profit(profit: f64) -> SqliteResult<()>`
  - Add profit/loss from a trade
  
- `reset_profit() -> SqliteResult<()>`
  - Reset all profit data to zero
  
- `get_profit_summary() -> SqliteResult<String>`
  - Get formatted profit summary string

### ProfitStats

#### Fields

- `total_profit: f64` - Total accumulated profit/loss
- `total_trades: i32` - Total number of trades
- `winning_trades: i32` - Number of profitable trades
- `losing_trades: i32` - Number of losing trades
- `largest_win: f64` - Largest single profit
- `largest_loss: f64` - Largest single loss
- `updated_at: String` - Last update timestamp

#### Methods

- `formatted_profit() -> String` - Get formatted profit string
- `is_profitable() -> bool` - Check if total profit is positive
- `win_rate() -> f64` - Calculate win rate percentage

## Example Usage

Run the example binary to see the profit database in action:

```bash
cargo run --bin profit_example
```

This will demonstrate:
1. Initializing the database
2. Getting current profit data
3. Adding sample profits/losses
4. Displaying updated summaries
5. How to reset profit data

## Integration with Main Bot

The profit database is automatically initialized in `main.rs` and can be used throughout the application. You can access it by:

1. Adding it to your module's dependencies
2. Using it in trade execution functions to record profits
3. Creating commands to display profit statistics
4. Adding profit tracking to notifications

### Telegram Bot Integration

The profit database is fully integrated with the Telegram bot control panel:

- **Remote Profit Reset**: Use `/reset` command to reset profit data
- **Profit Statistics**: Use `/profit` command to view current statistics
- **Status Monitoring**: Use `/status` command to see bot status and profit summary

See `TELEGRAM_BOT_README.md` for complete setup and usage instructions.

## Error Handling

All database operations return `SqliteResult<T>` which should be handled appropriately:

```rust
match profit_db.get_profit() {
    Ok(stats) => {
        // Handle successful result
        println!("Profit: {:.4} SOL", stats.total_profit);
    }
    Err(e) => {
        // Handle error
        tracing::error!("Failed to get profit: {}", e);
    }
}
```

## Testing

Run the tests to verify the database functionality:

```bash
cargo test profit_db
```

The tests verify:
- Database initialization
- Adding profits and losses
- Getting profit statistics
- Resetting profit data
- Edge cases and error handling 