use std::fs::File;
use std::io::Write;

fn main() {
    let mut file = File::create("test-data/valid_medium.csv").expect("Failed to create file");

    writeln!(file, "symbol,timestamp,bid,ask,bid_size,ask_size").expect("Failed to write header");

    let symbols = vec!["EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD"];
    let base_prices = vec![1.0921, 1.2500, 141.500, 0.6850, 1.3200];

    // Start at Unix timestamp 1704067200 (2024-01-01 00:00:00 UTC)
    let mut timestamp = 1704067200i64;

    for i in 0..10000 {
        let symbol_idx = i % symbols.len();
        let symbol = symbols[symbol_idx];
        let base_price = base_prices[symbol_idx];

        // Add some variation to prices
        let variation = ((i as f64 * 0.001).sin() * 0.0005).abs();
        let bid = base_price + variation;
        let ask = bid + 0.0002;

        let bid_size = 1000000 + (i * 100000) % 2000000;
        let ask_size = 1000000 + (i * 150000) % 2500000;

        writeln!(
            file,
            "{},{},{:.4},{:.4},{},{}",
            symbol, timestamp, bid, ask, bid_size, ask_size
        )
        .expect("Failed to write row");

        // Advance timestamp by 1 second
        timestamp += 1;
    }

    println!("Generated test-data/valid_medium.csv with 10,000 rows");
}
