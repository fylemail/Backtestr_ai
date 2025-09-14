use super::error::Result;
use rusqlite::Connection;

const TICK_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS ticks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    bid REAL NOT NULL,
    ask REAL NOT NULL,
    bid_size INTEGER,
    ask_size INTEGER,
    UNIQUE(symbol, timestamp)
)"#;

const TICK_INDEX_SCHEMA: &str = r#"
CREATE INDEX IF NOT EXISTS idx_ticks_timestamp
ON ticks(timestamp)
"#;

const BAR_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS bars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    timestamp_start INTEGER NOT NULL,
    timestamp_end INTEGER NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume INTEGER,
    tick_count INTEGER,
    created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000),
    UNIQUE(symbol, timeframe, timestamp_start)
)"#;

const BAR_INDEX_SCHEMA: &str = r#"
CREATE INDEX IF NOT EXISTS idx_bars_symbol_timeframe_timestamp
ON bars(symbol, timeframe, timestamp_start DESC)
"#;

const VERSION_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS db_version (
    version INTEGER PRIMARY KEY,
    migrated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
)"#;

pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Create version table first
    conn.execute(VERSION_TABLE_SCHEMA, [])?;

    // Check current version
    let current_version: Option<i32> = conn
        .query_row("SELECT MAX(version) FROM db_version", [], |row| row.get(0))
        .unwrap_or(None);

    // Create tick tables (version 1)
    if current_version.is_none() || current_version.unwrap() < 1 {
        conn.execute(TICK_TABLE_SCHEMA, [])?;
        conn.execute(TICK_INDEX_SCHEMA, [])?;
        conn.execute("INSERT OR IGNORE INTO db_version (version) VALUES (1)", [])?;
    }

    // Create bar tables (version 2)
    if current_version.is_none() || current_version.unwrap() < 2 {
        conn.execute(BAR_TABLE_SCHEMA, [])?;
        conn.execute(BAR_INDEX_SCHEMA, [])?;
        conn.execute("INSERT OR IGNORE INTO db_version (version) VALUES (2)", [])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_initialization() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn)?;

        // Check ticks table exists
        let ticks_table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='ticks'",
            [],
            |row| row.get(0),
        )?;
        assert!(ticks_table_exists);

        // Check bars table exists
        let bars_table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='bars'",
            [],
            |row| row.get(0),
        )?;
        assert!(bars_table_exists);

        // Check version table exists and has correct version
        let version: i32 =
            conn.query_row("SELECT MAX(version) FROM db_version", [], |row| row.get(0))?;
        assert_eq!(version, 2);

        Ok(())
    }

    #[test]
    fn test_schema_migration() -> Result<()> {
        let conn = Connection::open_in_memory()?;

        // First create only version 1 schema
        conn.execute(VERSION_TABLE_SCHEMA, [])?;
        conn.execute(TICK_TABLE_SCHEMA, [])?;
        conn.execute(TICK_INDEX_SCHEMA, [])?;
        conn.execute("INSERT INTO db_version (version) VALUES (1)", [])?;

        // Now run full initialization which should add version 2
        initialize_schema(&conn)?;

        // Check both tables exist
        let bars_table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='bars'",
            [],
            |row| row.get(0),
        )?;
        assert!(bars_table_exists);

        Ok(())
    }
}
