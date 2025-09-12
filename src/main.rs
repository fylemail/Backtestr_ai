mod config;
mod credentials;

use anyhow::Result;
use tracing::info;
use tracing_subscriber;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting BackTestr AI...");

    // Load configuration
    let config = config::Config::load()?;
    info!("Configuration loaded: {:?}", config.environment);

    println!("BackTestr AI - Multi-Timeframe Forex Backtesting Platform");
    println!("Version: 0.1.0");
    println!("Environment: {:?}", config.environment);

    Ok(())
}
