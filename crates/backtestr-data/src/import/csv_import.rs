use anyhow::{Context, Result};
use csv::Reader;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use super::validator::{validate_tick_data, ValidationError};
use crate::database::Database;
use crate::models::Tick;

const BATCH_SIZE: usize = 1000;
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("File too large: {0} bytes (max: {MAX_FILE_SIZE} bytes)")]
    FileTooLarge(u64),

    #[error("Line {line}: {error}")]
    ParseError { line: usize, error: String },

    #[error("Line {line}: validation failed: {error}")]
    ValidationError { line: usize, error: ValidationError },

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
}

#[derive(Debug, Clone)]
pub struct ImportSummary {
    pub file_path: PathBuf,
    pub total_rows: usize,
    pub rows_imported: usize,
    pub rows_skipped: usize,
    pub errors: Vec<String>,
    pub duration: Duration,
}

impl ImportSummary {
    pub fn success_rate(&self) -> f64 {
        if self.total_rows == 0 {
            0.0
        } else {
            (self.rows_imported as f64 / self.total_rows as f64) * 100.0
        }
    }
}

#[derive(Debug, Deserialize)]
struct CsvRow {
    symbol: String,
    timestamp: String,
    bid: f64,
    ask: f64,
    #[serde(default)]
    bid_size: Option<i64>,
    #[serde(default)]
    ask_size: Option<i64>,
}

pub struct CsvImporter {
    database: Database,
}

impl CsvImporter {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn import_file(&mut self, path: &Path) -> Result<ImportSummary> {
        let start_time = Instant::now();

        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(ImportError::FileTooLarge(metadata.len()).into());
        }

        info!("Starting CSV import from: {}", path.display());

        let file =
            File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

        let mut reader = Reader::from_reader(file);
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut total_rows = 0;
        let mut rows_imported = 0;
        let mut rows_skipped = 0;
        let mut errors = Vec::new();

        for (line_num, result) in reader.deserialize::<CsvRow>().enumerate() {
            total_rows += 1;
            let line = line_num + 2; // Account for header and 1-based indexing

            match result {
                Ok(row) => {
                    // Validate the data
                    if let Err(e) = validate_tick_data(
                        Some(&row.symbol),
                        Some(&row.timestamp),
                        Some(row.bid),
                        Some(row.ask),
                    ) {
                        warn!("Line {}: Validation failed: {}", line, e);
                        errors.push(format!("Line {}: {}", line, e));
                        rows_skipped += 1;
                        continue;
                    }

                    // Parse timestamp
                    let timestamp = match parse_timestamp(&row.timestamp) {
                        Ok(ts) => ts,
                        Err(e) => {
                            warn!(
                                "Line {}: Invalid timestamp '{}': {}",
                                line, row.timestamp, e
                            );
                            errors.push(format!("Line {}: Invalid timestamp: {}", line, e));
                            rows_skipped += 1;
                            continue;
                        }
                    };

                    // Create tick
                    let tick = Tick {
                        id: None,
                        symbol: row.symbol,
                        timestamp,
                        bid: row.bid,
                        ask: row.ask,
                        bid_size: row.bid_size,
                        ask_size: row.ask_size,
                    };

                    batch.push(tick);

                    // Process batch when it reaches BATCH_SIZE
                    if batch.len() >= BATCH_SIZE {
                        match self.database.insert_batch(&batch) {
                            Ok(_) => {
                                rows_imported += batch.len();
                                debug!("Imported batch of {} ticks", batch.len());
                            }
                            Err(e) => {
                                error!("Failed to insert batch: {}", e);
                                errors.push(format!("Batch insert failed at line {}: {}", line, e));
                                rows_skipped += batch.len();
                            }
                        }
                        batch.clear();
                    }
                }
                Err(e) => {
                    warn!("Line {}: Failed to parse CSV row: {}", line, e);
                    errors.push(format!("Line {}: Parse error: {}", line, e));
                    rows_skipped += 1;
                }
            }

            // Log progress every 10000 rows
            if total_rows % 10000 == 0 {
                info!("Processed {} rows...", total_rows);
            }
        }

        // Process remaining batch
        if !batch.is_empty() {
            match self.database.insert_batch(&batch) {
                Ok(_) => {
                    rows_imported += batch.len();
                    debug!("Imported final batch of {} ticks", batch.len());
                }
                Err(e) => {
                    error!("Failed to insert final batch: {}", e);
                    errors.push(format!("Final batch insert failed: {}", e));
                    rows_skipped += batch.len();
                }
            }
        }

        let duration = start_time.elapsed();

        let summary = ImportSummary {
            file_path: path.to_path_buf(),
            total_rows,
            rows_imported,
            rows_skipped,
            errors: errors.into_iter().take(100).collect(), // Limit errors to first 100
            duration,
        };

        info!(
            "Import completed: {} rows imported, {} skipped ({}% success rate) in {:?}",
            summary.rows_imported,
            summary.rows_skipped,
            summary.success_rate(),
            summary.duration
        );

        Ok(summary)
    }
}

fn parse_timestamp(timestamp_str: &str) -> Result<i64> {
    // Try parsing as ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
        return Ok(dt.timestamp_millis());
    }

    // Try parsing as Unix timestamp (seconds)
    if let Ok(ts) = timestamp_str.parse::<i64>() {
        // Assume timestamps after year 2000 and before 2100
        if ts > 946_684_800 && ts < 4_102_444_800 {
            return Ok(ts * 1000); // Convert to milliseconds
        }
        // Maybe it's already in milliseconds
        if ts > 946_684_800_000 && ts < 4_102_444_800_000 {
            return Ok(ts);
        }
    }

    anyhow::bail!("Unsupported timestamp format: {}", timestamp_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_db() -> Database {
        Database::new_memory().expect("Failed to create test database")
    }

    fn create_csv_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to temp file");
        file
    }

    #[test]
    fn test_parse_timestamp_iso8601() {
        let ts = parse_timestamp("2024-01-01T00:00:00Z").unwrap();
        assert_eq!(ts, 1704067200000);
    }

    #[test]
    fn test_parse_timestamp_unix_seconds() {
        let ts = parse_timestamp("1704067200").unwrap();
        assert_eq!(ts, 1704067200000);
    }

    #[test]
    fn test_parse_timestamp_unix_millis() {
        let ts = parse_timestamp("1704067200000").unwrap();
        assert_eq!(ts, 1704067200000);
    }

    #[test]
    fn test_import_valid_csv() {
        let csv_content = r#"symbol,timestamp,bid,ask,bid_size,ask_size
EURUSD,2024-01-01T00:00:00Z,1.0921,1.0923,1000000,1000000
EURUSD,2024-01-01T00:00:01Z,1.0922,1.0924,500000,750000
EURUSD,2024-01-01T00:00:02Z,1.0920,1.0922,2000000,1500000"#;

        let csv_file = create_csv_file(csv_content);
        let db = create_test_db();
        let mut importer = CsvImporter::new(db);

        let summary = importer.import_file(csv_file.path()).unwrap();

        assert_eq!(summary.total_rows, 3);
        assert_eq!(summary.rows_imported, 3);
        assert_eq!(summary.rows_skipped, 0);
        assert_eq!(summary.errors.len(), 0);
        assert_eq!(summary.success_rate(), 100.0);
    }

    #[test]
    fn test_import_with_invalid_rows() {
        let csv_content = r#"symbol,timestamp,bid,ask
EURUSD,2024-01-01T00:00:00Z,1.0921,1.0923
EURUSD,invalid-timestamp,1.0922,1.0924
EURUSD,2024-01-01T00:00:02Z,-1.0920,1.0922
EURUSD,2024-01-01T00:00:03Z,1.0925,1.0927"#;

        let csv_file = create_csv_file(csv_content);
        let db = create_test_db();
        let mut importer = CsvImporter::new(db);

        let summary = importer.import_file(csv_file.path()).unwrap();

        assert_eq!(summary.total_rows, 4);
        assert_eq!(summary.rows_imported, 2);
        assert_eq!(summary.rows_skipped, 2);
        assert_eq!(summary.errors.len(), 2);
        assert_eq!(summary.success_rate(), 50.0);
    }

    #[test]
    fn test_import_empty_file() {
        let csv_content = "symbol,timestamp,bid,ask\n";

        let csv_file = create_csv_file(csv_content);
        let db = create_test_db();
        let mut importer = CsvImporter::new(db);

        let summary = importer.import_file(csv_file.path()).unwrap();

        assert_eq!(summary.total_rows, 0);
        assert_eq!(summary.rows_imported, 0);
        assert_eq!(summary.rows_skipped, 0);
    }
}
