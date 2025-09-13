use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("DuckDB error: {0}")]
    DuckDB(#[from] duckdb::Error),

    #[error("Failed to initialize database: {0}")]
    InitializationError(String),

    #[error("Failed to execute query: {0}")]
    QueryError(String),

    #[error("Failed to insert data: {0}")]
    InsertError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;
