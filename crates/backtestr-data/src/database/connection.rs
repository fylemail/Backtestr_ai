use std::path::Path;
use duckdb::Connection;
use super::error::{DatabaseError, Result};
use super::schema::initialize_schema;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_database() -> Result<()> {
        let db = Database::new_memory()?;
        assert!(db.connection().is_autocommit());
        Ok(())
    }

    #[test]
    fn test_file_database() -> Result<()> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.duckdb");
        
        let db = Database::new_file(&db_path)?;
        assert!(db.connection().is_autocommit());
        assert!(db_path.exists());
        
        Ok(())
    }
}