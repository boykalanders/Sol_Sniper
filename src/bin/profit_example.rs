use crate::profit_db::ProfitDatabase;

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