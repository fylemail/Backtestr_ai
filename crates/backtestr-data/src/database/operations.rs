use super::connection::Database;
use super::error::{DatabaseError, Result};
use crate::models::{Bar, Tick};
use backtestr_core::Timeframe;
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::str::FromStr;

impl Database {
    pub fn insert_tick(&self, tick: &Tick) -> Result<()> {
        let sql = "INSERT INTO ticks (symbol, timestamp, bid, ask, bid_size, ask_size)
                   VALUES (?, ?, ?, ?, ?, ?)";

        self.connection()
            .execute(
                sql,
                params![
                    tick.symbol,
                    tick.timestamp,
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
                tick.timestamp,
                tick.bid,
                tick.ask,
                tick.bid_size,
                tick.ask_size
            ])
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;
        }

        Ok(())
    }

    pub fn insert_batch(&mut self, ticks: &[Tick]) -> Result<()> {
        // Use transaction for batch insert performance
        let conn = self.connection_mut();
        let tx = conn
            .transaction()
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

        {
            let sql =
                "INSERT OR IGNORE INTO ticks (symbol, timestamp, bid, ask, bid_size, ask_size)
                       VALUES (?, ?, ?, ?, ?, ?)";

            let mut stmt = tx
                .prepare(sql)
                .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

            for tick in ticks {
                stmt.execute(params![
                    tick.symbol,
                    tick.timestamp,
                    tick.bid,
                    tick.ask,
                    tick.bid_size,
                    tick.ask_size
                ])
                .map_err(|e| DatabaseError::InsertError(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

        Ok(())
    }

    pub fn query_ticks(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Tick>> {
        let sql = "SELECT id, symbol, timestamp, bid, ask, bid_size, ask_size
                   FROM ticks
                   WHERE symbol = ? AND timestamp >= ? AND timestamp <= ?
                   ORDER BY timestamp";

        let mut stmt = self
            .connection()
            .prepare(sql)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let ticks = stmt
            .query_map(
                params![symbol, start.timestamp_millis(), end.timestamp_millis()],
                |row| {
                    Ok(Tick {
                        id: row.get(0)?,
                        symbol: row.get(1)?,
                        timestamp: row.get(2)?,
                        bid: row.get(3)?,
                        ask: row.get(4)?,
                        bid_size: row.get(5)?,
                        ask_size: row.get(6)?,
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
                params![start.timestamp_millis(), end.timestamp_millis()],
            )
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(count)
    }

    // Bar operations

    pub fn insert_bar(&self, bar: &Bar) -> Result<()> {
        let sql = "INSERT OR REPLACE INTO bars
                   (symbol, timeframe, timestamp_start, timestamp_end,
                    open, high, low, close, volume, tick_count)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        self.connection()
            .execute(
                sql,
                params![
                    bar.symbol,
                    bar.timeframe.as_str(),
                    bar.timestamp_start,
                    bar.timestamp_end,
                    bar.open,
                    bar.high,
                    bar.low,
                    bar.close,
                    bar.volume,
                    bar.tick_count
                ],
            )
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

        Ok(())
    }

    pub fn batch_insert_bars(&mut self, bars: &[Bar]) -> Result<()> {
        let conn = self.connection_mut();
        let tx = conn
            .transaction()
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

        {
            let sql = "INSERT OR REPLACE INTO bars
                       (symbol, timeframe, timestamp_start, timestamp_end,
                        open, high, low, close, volume, tick_count)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

            let mut stmt = tx
                .prepare(sql)
                .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

            for bar in bars {
                stmt.execute(params![
                    bar.symbol,
                    bar.timeframe.as_str(),
                    bar.timestamp_start,
                    bar.timestamp_end,
                    bar.open,
                    bar.high,
                    bar.low,
                    bar.close,
                    bar.volume,
                    bar.tick_count
                ])
                .map_err(|e| DatabaseError::InsertError(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| DatabaseError::InsertError(e.to_string()))?;

        Ok(())
    }

    pub fn query_bars(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Bar>> {
        let sql = "SELECT id, symbol, timeframe, timestamp_start, timestamp_end,
                   open, high, low, close, volume, tick_count
                   FROM bars
                   WHERE symbol = ? AND timeframe = ?
                   AND timestamp_start >= ? AND timestamp_start <= ?
                   ORDER BY timestamp_start";

        let mut stmt = self
            .connection()
            .prepare(sql)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let bars = stmt
            .query_map(
                params![
                    symbol,
                    timeframe.as_str(),
                    start.timestamp_millis(),
                    end.timestamp_millis()
                ],
                |row| {
                    let timeframe_str: String = row.get(2)?;
                    let timeframe = Timeframe::from_str(&timeframe_str).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                        )
                    })?;

                    Ok(Bar {
                        id: row.get(0)?,
                        symbol: row.get(1)?,
                        timeframe,
                        timestamp_start: row.get(3)?,
                        timestamp_end: row.get(4)?,
                        open: row.get(5)?,
                        high: row.get(6)?,
                        low: row.get(7)?,
                        close: row.get(8)?,
                        volume: row.get(9)?,
                        tick_count: row.get(10)?,
                    })
                },
            )
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut result = Vec::new();
        for bar in bars {
            result.push(bar.map_err(|e| DatabaseError::QueryError(e.to_string()))?);
        }

        Ok(result)
    }

    pub fn get_latest_bar(&self, symbol: &str, timeframe: Timeframe) -> Result<Option<Bar>> {
        let sql = "SELECT id, symbol, timeframe, timestamp_start, timestamp_end,
                   open, high, low, close, volume, tick_count
                   FROM bars
                   WHERE symbol = ? AND timeframe = ?
                   ORDER BY timestamp_start DESC
                   LIMIT 1";

        let mut stmt = self
            .connection()
            .prepare(sql)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut bars = stmt
            .query_map(params![symbol, timeframe.as_str()], |row| {
                let timeframe_str: String = row.get(2)?;
                let timeframe = Timeframe::from_str(&timeframe_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                    )
                })?;

                Ok(Bar {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    timeframe,
                    timestamp_start: row.get(3)?,
                    timestamp_end: row.get(4)?,
                    open: row.get(5)?,
                    high: row.get(6)?,
                    low: row.get(7)?,
                    close: row.get(8)?,
                    volume: row.get(9)?,
                    tick_count: row.get(10)?,
                })
            })
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match bars.next() {
            Some(bar) => Ok(Some(
                bar.map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    pub fn delete_bars_by_symbol_timeframe(
        &self,
        symbol: &str,
        timeframe: Timeframe,
    ) -> Result<usize> {
        let count = self
            .connection()
            .execute(
                "DELETE FROM bars WHERE symbol = ? AND timeframe = ?",
                params![symbol, timeframe.as_str()],
            )
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(count)
    }

    pub fn count_bars(&self) -> Result<usize> {
        let count: i64 = self
            .connection()
            .query_row("SELECT COUNT(*) FROM bars", [], |row| row.get(0))
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(count as usize)
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

    #[test]
    fn test_insert_single_bar() -> Result<()> {
        let db = Database::new_memory()?;
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0918,
            1.0923,
        );

        db.insert_bar(&bar)?;

        let count = db.count_bars()?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[test]
    fn test_batch_insert_bars() -> Result<()> {
        let mut db = Database::new_memory()?;
        let bars = vec![
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                1704067200000,
                1704067260000,
                1.0920,
                1.0925,
                1.0918,
                1.0923,
            ),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                1704067260000,
                1704067320000,
                1.0923,
                1.0927,
                1.0921,
                1.0926,
            ),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M5,
                1704067200000,
                1704067500000,
                1.0920,
                1.0927,
                1.0918,
                1.0926,
            ),
        ];

        db.batch_insert_bars(&bars)?;

        let count = db.count_bars()?;
        assert_eq!(count, 3);

        Ok(())
    }

    #[test]
    fn test_query_bars_by_timeframe() -> Result<()> {
        let mut db = Database::new_memory()?;
        let base_time = 1704067200000; // 2024-01-01 00:00:00
        let bars = vec![
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                base_time,
                base_time + 60000,
                1.0920,
                1.0925,
                1.0918,
                1.0923,
            ),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                base_time + 60000,
                base_time + 120000,
                1.0923,
                1.0927,
                1.0921,
                1.0926,
            ),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M5,
                base_time,
                base_time + 300000,
                1.0920,
                1.0930,
                1.0918,
                1.0928,
            ),
        ];

        db.batch_insert_bars(&bars)?;

        let start = DateTime::from_timestamp_millis(base_time - 1000).unwrap();
        let end = DateTime::from_timestamp_millis(base_time + 400000).unwrap();

        let m1_bars = db.query_bars("EURUSD", Timeframe::M1, start, end)?;
        assert_eq!(m1_bars.len(), 2);

        let m5_bars = db.query_bars("EURUSD", Timeframe::M5, start, end)?;
        assert_eq!(m5_bars.len(), 1);

        Ok(())
    }

    #[test]
    fn test_get_latest_bar() -> Result<()> {
        let mut db = Database::new_memory()?;
        let base_time = 1704067200000;
        let bars = vec![
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                base_time,
                base_time + 60000,
                1.0920,
                1.0925,
                1.0918,
                1.0923,
            ),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                base_time + 60000,
                base_time + 120000,
                1.0923,
                1.0927,
                1.0921,
                1.0926,
            ),
        ];

        db.batch_insert_bars(&bars)?;

        let latest = db.get_latest_bar("EURUSD", Timeframe::M1)?;
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().timestamp_start, base_time + 60000);

        let no_bar = db.get_latest_bar("GBPUSD", Timeframe::M1)?;
        assert!(no_bar.is_none());

        Ok(())
    }

    #[test]
    fn test_delete_bars_by_symbol_timeframe() -> Result<()> {
        let mut db = Database::new_memory()?;
        let base_time = 1704067200000;
        let bars = vec![
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                base_time,
                base_time + 60000,
                1.0920,
                1.0925,
                1.0918,
                1.0923,
            ),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M5,
                base_time,
                base_time + 300000,
                1.0920,
                1.0930,
                1.0918,
                1.0928,
            ),
            Bar::new(
                "GBPUSD".to_string(),
                Timeframe::M1,
                base_time,
                base_time + 60000,
                1.2500,
                1.2505,
                1.2498,
                1.2503,
            ),
        ];

        db.batch_insert_bars(&bars)?;

        let deleted = db.delete_bars_by_symbol_timeframe("EURUSD", Timeframe::M1)?;
        assert_eq!(deleted, 1);

        let remaining = db.count_bars()?;
        assert_eq!(remaining, 2);

        Ok(())
    }
}
