use std::fs::File;
use std::io::Write;
use chrono::{DateTime, Duration, Utc};

fn main() {
    let mut file = File::create("test-data/valid_medium.csv").expect("Failed to create file");

    writeln!(file, "symbol,timestamp,bid,ask,bid_size,ask_size").expect("Failed to write header");

    let symbols = vec!["EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD"];
    let base_prices = vec![1.0921, 1.2500, 141.500, 0.6850, 1.3200];
    let mut timestamp = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    for i in 0..10000 {
        let symbol_idx = i % symbols.len();
        let symbol = symbols[symbol_idx];
        let base_price = base_prices[symbol_idx];

        // Add some random variation to prices
        let variation = ((i as f64).sin() * 0.0005).abs();
        let bid = base_price + variation;
        let ask = bid + 0.0002;

        let bid_size = 1000000 + (i * 100000) % 2000000;
        let ask_size = 1000000 + (i * 150000) % 2500000;

        writeln!(
            file,
            "{},{},{:.4},{:.4},{},{}",
            symbol,
            timestamp.to_rfc3339(),
            bid,
            ask,
            bid_size,
            ask_size
        ).expect("Failed to write row");

        // Advance timestamp by 1 second
        timestamp = timestamp + Duration::seconds(1);
    }

    println!("Generated test-data/valid_medium.csv with 10,000 rows");
}