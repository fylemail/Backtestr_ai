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

pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute(TICK_TABLE_SCHEMA, [])?;
    conn.execute(TICK_INDEX_SCHEMA, [])?;
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

        // SQLite uses sqlite_master instead of information_schema
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='ticks'",
            [],
            |row| row.get(0),
        )?;

        assert!(table_exists);
        Ok(())
    }
}
