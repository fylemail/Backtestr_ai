use anyhow::{Context, Result};
use backtestr_data::{CsvImporter, Database};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use comfy_table::{Cell, ContentArrangement, Table};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "backtestr")]
#[command(about = "BackTestr CLI - Query and manage tick data", long_about = None)]
struct Cli {
    /// Path to database file
    #[arg(long, default_value = "./backtest.sqlite")]
    db: PathBuf,

    /// Use in-memory database
    #[arg(long)]
    memory: bool,

    /// Enable verbose output
    #[arg(long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import tick data from CSV file
    Import {
        /// Path to CSV file
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Query tick data
    Query {
        /// Symbol to query (e.g., EURUSD)
        #[arg(short, long)]
        symbol: String,

        /// Start date (ISO format: 2024-01-01 or 2024-01-01T00:00:00Z)
        #[arg(long)]
        from: Option<String>,

        /// End date (ISO format: 2024-01-01 or 2024-01-01T00:00:00Z)
        #[arg(long)]
        to: Option<String>,

        /// Maximum number of results
        #[arg(long, default_value = "100")]
        limit: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Show database statistics
    Stats,

    /// Delete tick data
    Delete {
        /// Symbol to delete
        #[arg(short, long)]
        symbol: Option<String>,

        /// Delete data from this date
        #[arg(long)]
        from: Option<String>,

        /// Delete data to this date
        #[arg(long)]
        to: Option<String>,

        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Csv,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    if cli.verbose {
        tracing_subscriber::fmt().with_env_filter("debug").init();
    }

    match &cli.command {
        Commands::Import { file } => handle_import(&cli, file),
        Commands::Query {
            symbol,
            from,
            to,
            limit,
            format,
        } => {
            let database = create_database(&cli)?;
            handle_query(
                &database,
                symbol,
                from.clone(),
                to.clone(),
                *limit,
                format.clone(),
            )
        }
        Commands::Stats => {
            let database = create_database(&cli)?;
            handle_stats(&database)
        }
        Commands::Delete {
            symbol,
            from,
            to,
            confirm,
        } => {
            let database = create_database(&cli)?;
            handle_delete(
                &database,
                symbol.clone(),
                from.clone(),
                to.clone(),
                *confirm,
            )
        }
    }
}

fn create_database(cli: &Cli) -> Result<Database> {
    if cli.memory {
        Database::new_memory().context("Failed to create memory database")
    } else {
        Database::new_file(&cli.db).context("Failed to open database file")
    }
}

fn handle_import(cli: &Cli, file: &Path) -> Result<()> {
    println!("Importing data from: {}", file.display());

    // Create a fresh database connection for the importer
    let database = if cli.memory {
        Database::new_memory()?
    } else {
        Database::new_file(&cli.db)?
    };

    let mut importer = CsvImporter::new(database);
    let summary = importer
        .import_file(file)
        .context("Failed to import CSV file")?;

    println!("\n📊 Import Summary:");
    println!("  Total rows: {}", summary.total_rows);
    println!("  Imported: {}", summary.rows_imported);
    println!("  Skipped: {}", summary.rows_skipped);
    println!("  Success rate: {:.1}%", summary.success_rate());
    println!("  Duration: {:?}", summary.duration);

    if !summary.errors.is_empty() {
        println!("\n⚠️  Errors (first 10):");
        for error in summary.errors.iter().take(10) {
            println!("  - {}", error);
        }
    }

    Ok(())
}

fn handle_query(
    database: &Database,
    symbol: &str,
    from: Option<String>,
    to: Option<String>,
    limit: usize,
    format: OutputFormat,
) -> Result<()> {
    // Parse dates
    let start =
        parse_date(from.as_deref()).unwrap_or_else(|_| Utc::now() - chrono::Duration::days(30));
    let end = parse_date(to.as_deref()).unwrap_or_else(|_| Utc::now());

    // Query ticks
    let ticks = database
        .query_ticks(symbol, start, end)
        .context("Failed to query ticks")?;

    if ticks.is_empty() {
        println!(
            "No ticks found for {} between {} and {}",
            symbol, start, end
        );
        return Ok(());
    }

    // Limit results
    let ticks: Vec<_> = ticks.into_iter().take(limit).collect();

    // Format output
    match format {
        OutputFormat::Table => {
            let mut table = Table::new();
            table
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    "Symbol",
                    "Timestamp",
                    "Bid",
                    "Ask",
                    "Bid Size",
                    "Ask Size",
                ]);

            for tick in &ticks {
                let timestamp = DateTime::from_timestamp_millis(tick.timestamp)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| tick.timestamp.to_string());

                table.add_row(vec![
                    Cell::new(&tick.symbol),
                    Cell::new(timestamp),
                    Cell::new(format!("{:.5}", tick.bid)),
                    Cell::new(format!("{:.5}", tick.ask)),
                    Cell::new(tick.bid_size.map_or("".to_string(), |s| s.to_string())),
                    Cell::new(tick.ask_size.map_or("".to_string(), |s| s.to_string())),
                ]);
            }

            println!("{table}");
            println!("\nShowing {} of {} total results", ticks.len(), ticks.len());
        }
        OutputFormat::Csv => {
            println!("symbol,timestamp,bid,ask,bid_size,ask_size");
            for tick in &ticks {
                println!(
                    "{},{},{},{},{},{}",
                    tick.symbol,
                    tick.timestamp,
                    tick.bid,
                    tick.ask,
                    tick.bid_size.unwrap_or(0),
                    tick.ask_size.unwrap_or(0)
                );
            }
        }
        OutputFormat::Json => {
            // For now, print a simple JSON-like format
            // serde_json is already in workspace dependencies
            println!("[");
            for (i, tick) in ticks.iter().enumerate() {
                println!("  {{");
                println!("    \"symbol\": \"{}\",", tick.symbol);
                println!("    \"timestamp\": {},", tick.timestamp);
                println!("    \"bid\": {},", tick.bid);
                println!("    \"ask\": {},", tick.ask);
                println!("    \"bid_size\": {},", tick.bid_size.unwrap_or(0));
                println!("    \"ask_size\": {}", tick.ask_size.unwrap_or(0));
                if i < ticks.len() - 1 {
                    println!("  }},");
                } else {
                    println!("  }}");
                }
            }
            println!("]");
        }
    }

    Ok(())
}

fn handle_stats(database: &Database) -> Result<()> {
    let total_ticks = database.count_ticks()?;

    println!("📊 Database Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━");
    println!("Total ticks: {}", total_ticks);

    // Get unique symbols and their counts
    // For now, we'll just show total count
    // In a real implementation, we'd add a method to get symbol statistics

    Ok(())
}

fn handle_delete(
    database: &Database,
    symbol: Option<String>,
    from: Option<String>,
    to: Option<String>,
    confirm: bool,
) -> Result<()> {
    if !confirm {
        println!("❌ Deletion requires --confirm flag for safety");
        return Ok(());
    }

    let deleted = if let Some(symbol) = symbol {
        println!("Deleting all ticks for symbol: {}", symbol);
        database.delete_ticks_by_symbol(&symbol)?
    } else if let (Some(from), Some(to)) = (from, to) {
        let start = parse_date(Some(&from))?;
        let end = parse_date(Some(&to))?;
        println!("Deleting ticks from {} to {}", start, end);
        database.delete_ticks_by_time_range(start, end)?
    } else {
        println!("❌ Please specify either --symbol or both --from and --to");
        return Ok(());
    };

    println!("✅ Deleted {} ticks", deleted);
    Ok(())
}

fn parse_date(date_str: Option<&str>) -> Result<DateTime<Utc>> {
    if let Some(date) = date_str {
        // Try parsing as full ISO 8601
        if let Ok(dt) = DateTime::parse_from_rfc3339(date) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try parsing as date only (add time)
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            let datetime = dt
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
            return Ok(DateTime::from_naive_utc_and_offset(datetime, Utc));
        }

        anyhow::bail!("Invalid date format: {}. Use YYYY-MM-DD or ISO 8601", date)
    } else {
        anyhow::bail!("No date provided")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_date() {
        // Test ISO 8601
        let date = parse_date(Some("2024-01-01T12:00:00Z")).unwrap();
        assert_eq!(date.year(), 2024);

        // Test date only
        let date = parse_date(Some("2024-01-01")).unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.hour(), 0);
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
