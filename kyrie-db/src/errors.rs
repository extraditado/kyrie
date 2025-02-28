use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection Error: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("Transaction error: {0}")]
    TransactionError(String),
}
