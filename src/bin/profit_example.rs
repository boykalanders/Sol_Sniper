use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;
use std::sync::Mutex;

struct ProfitDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl ProfitDatabase {
    /// Initialize the profit database
    fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let conn = Arc::new(Mutex::new(conn));
        let db = ProfitDatabase { conn };
        db.init_table()?;
        Ok(db)
    }

    /// Initialize the profit table if it doesn't exist
    fn init_table(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS profit_tracking (
                id INTEGER PRIMARY KEY,
                total_profit REAL NOT NULL DEFAULT 0.0,
                total_trades INTEGER NOT NULL DEFAULT 0,
                winning_trades INTEGER NOT NULL DEFAULT 0,
                losing_trades INTEGER NOT NULL DEFAULT 0,
                largest_win REAL NOT NULL DEFAULT 0.0,
                largest_loss REAL NOT NULL DEFAULT 0.0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Insert initial record if table is empty
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM profit_tracking", [], |row| row.get(0))?;
        if count == 0 {
            conn.execute(
                "INSERT INTO profit_tracking (total_profit, total_trades, winning_trades, losing_trades, largest_win, largest_loss) 
                 VALUES (0.0, 0, 0, 0.0, 0.0)",
                [],
            )?;
        }

        Ok(())
    }

    /// Get current profit statistics
    fn get_profit(&self) -> SqliteResult<ProfitStats> {
        let conn = self.conn.lock().unwrap();
        let row = conn.query_row(
            "SELECT total_profit, total_trades, winning_trades, losing_trades, largest_win, largest_loss, updated_at 
             FROM profit_tracking ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                Ok(ProfitStats {
                    total_profit: row.get(0)?,
                    total_trades: row.get(1)?,
                    winning_trades: row.get(2)?,
                    losing_trades: row.get(3)?,
                    largest_win: row.get(4)?,
                    largest_loss: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )?;
        Ok(row)
    }

    /// Get profit as a simple f64 value (for backward compatibility)
    fn get_profit_value(&self) -> SqliteResult<f64> {
        let stats = self.get_profit()?;
        Ok(stats.total_profit)
    }

    /// Add profit from a trade
    fn add_profit(&self, profit: f64) -> SqliteResult<()> {
        let stats = self.get_profit()?;
        let new_total_profit = stats.total_profit + profit;
        let new_total_trades = stats.total_trades + 1;
        let new_winning_trades = if profit > 0.0 { stats.winning_trades + 1 } else { stats.winning_trades };
        let new_losing_trades = if profit < 0.0 { stats.losing_trades + 1 } else { stats.losing_trades };
        let new_largest_win = if profit > stats.largest_win { profit } else { stats.largest_win };
        let new_largest_loss = if profit < stats.largest_loss { profit } else { stats.largest_loss };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE profit_tracking SET 
                total_profit = ?, 
                total_trades = ?, 
                winning_trades = ?, 
                losing_trades = ?, 
                largest_win = ?, 
                largest_loss = ?, 
                updated_at = CURRENT_TIMESTAMP",
            rusqlite::params![
                new_total_profit,
                new_total_trades,
                new_winning_trades,
                new_losing_trades,
                new_largest_win,
                new_largest_loss,
            ],
        )?;
        Ok(())
    }

    /// Reset profit tracking
    fn _reset_profit(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE profit_tracking SET 
                total_profit = 0.0, 
                total_trades = 0, 
                winning_trades = 0, 
                losing_trades = 0, 
                largest_win = 0.0, 
                largest_loss = 0.0, 
                updated_at = CURRENT_TIMESTAMP",
            [],
        )?;
        Ok(())
    }

    /// Get a formatted profit summary
    fn get_profit_summary(&self) -> SqliteResult<String> {
        let stats = self.get_profit()?;
        let win_rate = if stats.total_trades > 0 {
            (stats.winning_trades as f64 / stats.total_trades as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(format!(
            "💰 **Profit Summary**\n\n\
            **Total Profit:** {:.4} SOL\n\
            **Total Trades:** {}\n\
            **Win Rate:** {:.1}%\n\
            **Winning Trades:** {}\n\
            **Losing Trades:** {}\n\
            **Largest Win:** {:.4} SOL\n\
            **Largest Loss:** {:.4} SOL\n\
            **Last Updated:** {}",
            stats.total_profit,
            stats.total_trades,
            win_rate,
            stats.winning_trades,
            stats.losing_trades,
            stats.largest_win,
            stats.largest_loss,
            stats.updated_at
        ))
    }
}

impl Clone for ProfitDatabase {
    fn clone(&self) -> Self {
        ProfitDatabase {
            conn: Arc::clone(&self.conn),
        }
    }
}

struct ProfitStats {
    total_profit: f64,
    total_trades: i32,
    winning_trades: i32,
    losing_trades: i32,
    largest_win: f64,
    largest_loss: f64,
    updated_at: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the profit database
    let profit_db = ProfitDatabase::new("profit_tracking.db")?;
    println!("✅ Profit database initialized");

    // Example 1: Get current profit
    println!("\n📊 Getting current profit...");
    match profit_db.get_profit() {
        Ok(stats) => {
            println!("💰 Current Profit: {:.4} SOL", stats.total_profit);
            println!("📈 Total Trades: {}", stats.total_trades);
            println!("✅ Winning Trades: {}", stats.winning_trades);
            println!("❌ Losing Trades: {}", stats.losing_trades);
            println!("🏆 Largest Win: {:.4} SOL", stats.largest_win);
            println!("💸 Largest Loss: {:.4} SOL", stats.largest_loss);
            println!("📅 Last Updated: {}", stats.updated_at);
        }
        Err(e) => println!("❌ Error getting profit: {}", e),
    }

    // Example 2: Add some sample profits
    println!("\n📈 Adding sample profits...");
    let sample_profits = vec![0.5, -0.2, 1.0, -0.1, 0.8];
    
    for profit in sample_profits {
        match profit_db.add_profit(profit) {
            Ok(_) => println!("✅ Added profit: {:.4} SOL", profit),
            Err(e) => println!("❌ Error adding profit: {}", e),
        }
    }

    // Example 3: Get updated profit summary
    println!("\n📊 Updated profit summary:");
    match profit_db.get_profit_summary() {
        Ok(summary) => println!("{}", summary),
        Err(e) => println!("❌ Error getting summary: {}", e),
    }

    // Example 4: Get simple profit value
    println!("\n💰 Simple profit value:");
    match profit_db.get_profit_value() {
        Ok(profit) => println!("Total Profit: {:.4} SOL", profit),
        Err(e) => println!("❌ Error getting profit value: {}", e),
    }

    // Example 5: Reset profit (commented out to avoid losing data)
    println!("\n🔄 Profit reset example (commented out to preserve data):");
    println!("To reset profit, uncomment the following line:");
    println!("// profit_db.reset_profit()?;");

    // Uncomment the line below to actually reset the profit
    // match profit_db.reset_profit() {
    //     Ok(_) => println!("✅ Profit reset successfully"),
    //     Err(e) => println!("❌ Error resetting profit: {}", e),
    // }

    println!("\n✅ Example completed successfully!");
    Ok(())
} 