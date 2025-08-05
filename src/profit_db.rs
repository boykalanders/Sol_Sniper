use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use tracing::{info, error};

pub struct ProfitDatabase {
    conn: Connection,
}

impl ProfitDatabase {
    /// Initialize the profit database
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = ProfitDatabase { conn };
        db.init_table()?;
        Ok(db)
    }

    /// Initialize the profit table if it doesn't exist
    fn init_table(&self) -> SqliteResult<()> {
        self.conn.execute(
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
        let count: i32 = self.conn.query_row("SELECT COUNT(*) FROM profit_tracking", [], |row| row.get(0))?;
        if count == 0 {
            self.conn.execute(
                "INSERT INTO profit_tracking (total_profit, total_trades, winning_trades, losing_trades, largest_win, largest_loss) 
                 VALUES (0.0, 0, 0, 0, 0.0, 0.0)",
                [],
            )?;
            info!("Initialized profit tracking database");
        }

        Ok(())
    }

    /// Get current profit statistics
    pub fn get_profit(&self) -> SqliteResult<ProfitStats> {
        let row = self.conn.query_row(
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
    pub fn get_profit_value(&self) -> SqliteResult<f64> {
        let stats = self.get_profit()?;
        Ok(stats.total_profit)
    }

    /// Add profit from a trade
    pub fn add_profit(&self, profit: f64) -> SqliteResult<()> {
        let stats = self.get_profit()?;
        let new_total_profit = stats.total_profit + profit;
        let new_total_trades = stats.total_trades + 1;
        let new_winning_trades = if profit > 0.0 { stats.winning_trades + 1 } else { stats.winning_trades };
        let new_losing_trades = if profit < 0.0 { stats.losing_trades + 1 } else { stats.losing_trades };
        let new_largest_win = if profit > stats.largest_win { profit } else { stats.largest_win };
        let new_largest_loss = if profit < stats.largest_loss { profit } else { stats.largest_loss };

        self.conn.execute(
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

        info!("Added profit: {:.4} SOL, Total: {:.4} SOL", profit, new_total_profit);
        Ok(())
    }

    /// Reset all profit data to zero
    pub fn reset_profit(&self) -> SqliteResult<()> {
        self.conn.execute(
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

        info!("Reset all profit data to zero");
        Ok(())
    }

    /// Get profit summary as a formatted string
    pub fn get_profit_summary(&self) -> SqliteResult<String> {
        let stats = self.get_profit()?;
        let win_rate = if stats.total_trades > 0 {
            (stats.winning_trades as f64 / stats.total_trades as f64) * 100.0
        } else {
            0.0
        };

        let summary = format!(
            "💰 Profit Summary:\n\
             Total Profit: {:.4} SOL\n\
             Total Trades: {}\n\
             Winning Trades: {}\n\
             Losing Trades: {}\n\
             Win Rate: {:.1}%\n\
             Largest Win: {:.4} SOL\n\
             Largest Loss: {:.4} SOL\n\
             Last Updated: {}",
            stats.total_profit,
            stats.total_trades,
            stats.winning_trades,
            stats.losing_trades,
            win_rate,
            stats.largest_win,
            stats.largest_loss,
            stats.updated_at
        );

        Ok(summary)
    }
}

#[derive(Debug)]
pub struct ProfitStats {
    pub total_profit: f64,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub updated_at: String,
}

impl ProfitStats {
    /// Get formatted profit string
    pub fn formatted_profit(&self) -> String {
        format!("{:.4} SOL", self.total_profit)
    }

    /// Check if profit is positive
    pub fn is_profitable(&self) -> bool {
        self.total_profit > 0.0
    }

    /// Get win rate as percentage
    pub fn win_rate(&self) -> f64 {
        if self.total_trades > 0 {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_profit_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_profit.db");
        
        // Test database creation
        let db = ProfitDatabase::new(db_path.to_str().unwrap()).unwrap();
        
        // Test initial state
        let stats = db.get_profit().unwrap();
        assert_eq!(stats.total_profit, 0.0);
        assert_eq!(stats.total_trades, 0);
        
        // Test adding profit
        db.add_profit(1.5).unwrap();
        let stats = db.get_profit().unwrap();
        assert_eq!(stats.total_profit, 1.5);
        assert_eq!(stats.total_trades, 1);
        assert_eq!(stats.winning_trades, 1);
        assert_eq!(stats.largest_win, 1.5);
        
        // Test adding loss
        db.add_profit(-0.5).unwrap();
        let stats = db.get_profit().unwrap();
        assert_eq!(stats.total_profit, 1.0);
        assert_eq!(stats.total_trades, 2);
        assert_eq!(stats.winning_trades, 1);
        assert_eq!(stats.losing_trades, 1);
        assert_eq!(stats.largest_loss, -0.5);
        
        // Test reset
        db.reset_profit().unwrap();
        let stats = db.get_profit().unwrap();
        assert_eq!(stats.total_profit, 0.0);
        assert_eq!(stats.total_trades, 0);
    }
} 