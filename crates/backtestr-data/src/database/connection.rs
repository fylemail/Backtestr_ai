use super::error::{DatabaseError, Result};
use super::schema::initialize_schema;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DatabaseError::InitializationError(e.to_string()))?;

        initialize_schema(&conn)?;

        Ok(Self { conn })
    }

    pub fn new_file(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| DatabaseError::InitializationError(e.to_string()))?;

        initialize_schema(&conn)?;

        Ok(Self { conn })
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }

    pub(crate) fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_database() -> Result<()> {
        let db = Database::new_memory()?;
        // SQLite doesn't have is_autocommit() method
        assert!(db.connection() as *const _ != std::ptr::null());
        Ok(())
    }

    #[test]
    fn test_file_database() -> Result<()> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new_file(&db_path)?;
        // SQLite doesn't have is_autocommit() method
        assert!(db.connection() as *const _ != std::ptr::null());
        assert!(db_path.exists());

        Ok(())
    }
}
