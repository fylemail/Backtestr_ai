use backtestr_data::{Database, Result, Tick};
use chrono::{Duration, Utc};
use tempfile::tempdir;

#[test]
fn test_full_lifecycle() -> Result<()> {
    let db = Database::new_memory()?;
    let now = Utc::now();

    // Create test ticks
    let ticks = vec![
        Tick::new(
            "EURUSD".to_string(),
            now - Duration::seconds(10),
            1.0920,
            1.0922,
        )
        .with_sizes(1000000, 1000000),
        Tick::new(
            "EURUSD".to_string(),
            now - Duration::seconds(5),
            1.0921,
            1.0923,
        )
        .with_sizes(1500000, 1500000),
        Tick::new("EURUSD".to_string(), now, 1.0922, 1.0924).with_sizes(2000000, 2000000),
        Tick::new("GBPUSD".to_string(), now, 1.2500, 1.2502).with_sizes(1000000, 1000000),
    ];

    // Insert ticks
    db.insert_ticks(&ticks)?;

    // Verify count
    let count = db.count_ticks()?;
    assert_eq!(count, 4);

    // Query specific symbol
    let eurusd_ticks = db.query_ticks(
        "EURUSD",
        now - Duration::seconds(20),
        now + Duration::seconds(10),
    )?;
    assert_eq!(eurusd_ticks.len(), 3);

    // Delete by symbol
    let deleted = db.delete_ticks_by_symbol("GBPUSD")?;
    assert_eq!(deleted, 1);

    // Verify final count
    let final_count = db.count_ticks()?;
    assert_eq!(final_count, 3);

    Ok(())
}

#[test]
fn test_file_persistence() -> Result<()> {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let now = Utc::now();

    // Create and populate database
    {
        let db = Database::new_file(&db_path)?;
        let tick = Tick::new("EURUSD".to_string(), now, 1.0921, 1.0923);
        db.insert_tick(&tick)?;
    }

    // Reopen and verify data persisted
    {
        let db = Database::new_file(&db_path)?;
        let count = db.count_ticks()?;
        assert_eq!(count, 1);

        let ticks = db.query_ticks(
            "EURUSD",
            now - Duration::seconds(10),
            now + Duration::seconds(10),
        )?;
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].symbol, "EURUSD");
    }

    Ok(())
}

#[test]
fn test_concurrent_access() -> Result<()> {
    use std::thread;

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("concurrent.db");

    // Create database
    let db = Database::new_file(&db_path)?;
    let now = Utc::now();

    // Insert initial data
    let tick = Tick::new("EURUSD".to_string(), now, 1.0921, 1.0923);
    db.insert_tick(&tick)?;

    // Note: SQLite with WAL mode handles concurrent reads well
    // This test verifies basic concurrent read operations work

    let handles: Vec<_> = (0..3)
        .map(|_i| {
            let path = db_path.clone();
            thread::spawn(move || -> Result<()> {
                let db = Database::new_file(&path)?;
                let count = db.count_ticks()?;
                assert!(count > 0);
                Ok(())
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}
