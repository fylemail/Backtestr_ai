use super::connection::Database;
use super::error::{DatabaseError, Result};
use crate::models::Tick;
use chrono::{DateTime, Utc};
use rusqlite::params;

impl Database {
    pub fn insert_tick(&self, tick: &Tick) -> Result<()> {
        let sql = "INSERT INTO ticks (symbol, timestamp, bid, ask, bid_size, ask_size) 
                   VALUES (?, ?, ?, ?, ?, ?)";

        self.connection()
            .execute(
                sql,
                params![
                    tick.symbol,
                    tick.timestamp.to_rfc3339(),
                    tick.bid,
                    tick.ask,
                    tick.bid_size,
                    tick.ask_size
                ],
            )
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

        Ok(())
    }

    pub fn insert_ticks(&self, ticks: &[Tick]) -> Result<()> {
        // Use prepared statements for batch insert
        let sql = "INSERT INTO ticks (symbol, timestamp, bid, ask, bid_size, ask_size) 
                   VALUES (?, ?, ?, ?, ?, ?)";

        let mut stmt = self
            .connection()
            .prepare(sql)
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

        for tick in ticks {
            stmt.execute(params![
                tick.symbol,
                tick.timestamp.to_rfc3339(),
                tick.bid,
                tick.ask,
                tick.bid_size,
                tick.ask_size
            ])
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;
        }

        Ok(())
    }

    pub fn query_ticks(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Tick>> {
        let sql = "SELECT symbol, timestamp, bid, ask, bid_size, ask_size 
                   FROM ticks 
                   WHERE symbol = ? AND timestamp >= ? AND timestamp <= ?
                   ORDER BY timestamp";

        let mut stmt = self
            .connection()
            .prepare(sql)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let ticks = stmt
            .query_map(
                params![symbol, start.to_rfc3339(), end.to_rfc3339()],
                |row| {
                    let timestamp_str: String = row.get(1)?;
                    let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                1,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                        .with_timezone(&Utc);

                    Ok(Tick {
                        symbol: row.get(0)?,
                        timestamp,
                        bid: row.get(2)?,
                        ask: row.get(3)?,
                        bid_size: row.get(4)?,
                        ask_size: row.get(5)?,
                    })
                },
            )
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut result = Vec::new();
        for tick in ticks {
            result.push(tick.map_err(|e| DatabaseError::QueryError(e.to_string()))?);
        }

        Ok(result)
    }

    pub fn count_ticks(&self) -> Result<usize> {
        let count: i64 = self
            .connection()
            .query_row("SELECT COUNT(*) FROM ticks", [], |row| row.get(0))
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(count as usize)
    }

    pub fn delete_ticks_by_symbol(&self, symbol: &str) -> Result<usize> {
        let count = self
            .connection()
            .execute("DELETE FROM ticks WHERE symbol = ?", params![symbol])
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(count)
    }

    pub fn delete_ticks_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<usize> {
        let count = self
            .connection()
            .execute(
                "DELETE FROM ticks WHERE timestamp >= ? AND timestamp <= ?",
                params![start.to_rfc3339(), end.to_rfc3339()],
            )
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_tick(symbol: &str, offset_secs: i64) -> Tick {
        let timestamp = Utc::now() + Duration::seconds(offset_secs);
        Tick::new(symbol.to_string(), timestamp, 1.0921, 1.0923).with_sizes(1000000, 1000000)
    }

    #[test]
    fn test_insert_single_tick() -> Result<()> {
        let db = Database::new_memory()?;
        let tick = create_test_tick("EURUSD", 0);

        db.insert_tick(&tick)?;

        let count = db.count_ticks()?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[test]
    fn test_insert_batch_ticks() -> Result<()> {
        let db = Database::new_memory()?;
        let ticks = vec![
            create_test_tick("EURUSD", 0),
            create_test_tick("EURUSD", 1),
            create_test_tick("EURUSD", 2),
        ];

        db.insert_ticks(&ticks)?;

        let count = db.count_ticks()?;
        assert_eq!(count, 3);

        Ok(())
    }

    #[test]
    fn test_query_ticks_by_symbol_and_time() -> Result<()> {
        let db = Database::new_memory()?;
        let now = Utc::now();
        let ticks = vec![
            Tick::new(
                "EURUSD".to_string(),
                now - Duration::hours(2),
                1.0920,
                1.0922,
            ),
            Tick::new(
                "EURUSD".to_string(),
                now - Duration::hours(1),
                1.0921,
                1.0923,
            ),
            Tick::new("EURUSD".to_string(), now, 1.0922, 1.0924),
            Tick::new("GBPUSD".to_string(), now, 1.2500, 1.2502),
        ];

        db.insert_ticks(&ticks)?;

        let queried =
            db.query_ticks("EURUSD", now - Duration::hours(3), now + Duration::hours(1))?;

        assert_eq!(queried.len(), 3);
        assert!(queried.iter().all(|t| t.symbol == "EURUSD"));

        Ok(())
    }

    #[test]
    fn test_delete_ticks_by_symbol() -> Result<()> {
        let db = Database::new_memory()?;
        let ticks = vec![
            create_test_tick("EURUSD", 0),
            create_test_tick("EURUSD", 1),
            create_test_tick("GBPUSD", 0),
        ];

        db.insert_ticks(&ticks)?;

        let deleted = db.delete_ticks_by_symbol("EURUSD")?;
        assert_eq!(deleted, 2);

        let remaining = db.count_ticks()?;
        assert_eq!(remaining, 1);

        Ok(())
    }

    #[test]
    fn test_delete_ticks_by_time_range() -> Result<()> {
        let db = Database::new_memory()?;
        let now = Utc::now();
        let ticks = vec![
            Tick::new(
                "EURUSD".to_string(),
                now - Duration::hours(3),
                1.0920,
                1.0922,
            ),
            Tick::new(
                "EURUSD".to_string(),
                now - Duration::hours(1),
                1.0921,
                1.0923,
            ),
            Tick::new(
                "EURUSD".to_string(),
                now + Duration::hours(1),
                1.0922,
                1.0924,
            ),
        ];

        db.insert_ticks(&ticks)?;

        let deleted = db.delete_ticks_by_time_range(now - Duration::hours(2), now)?;
        assert_eq!(deleted, 1);

        let remaining = db.count_ticks()?;
        assert_eq!(remaining, 2);

        Ok(())
    }
}
