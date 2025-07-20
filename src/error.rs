
use thiserror::Error;

#[derive(Error,Debug)]
pub enum DatabaseError{
    #[error("Path Error")]
    Path,
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite Error: {0}")]
    Sqlite(#[from] rusqlite::Error)
}