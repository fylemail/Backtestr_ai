use duckdb::Connection;
use super::error::Result;

const TICK_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS ticks (
    symbol VARCHAR NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    bid DOUBLE PRECISION NOT NULL,
    ask DOUBLE PRECISION NOT NULL,
    bid_size INTEGER,
    ask_size INTEGER,
    PRIMARY KEY (symbol, timestamp)
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
    use duckdb::Connection;

    #[test]
    fn test_schema_initialization() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn)?;
        
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_name = 'ticks'",
            [],
            |row| row.get(0)
        )?;
        
        assert!(table_exists);
        Ok(())
    }
}