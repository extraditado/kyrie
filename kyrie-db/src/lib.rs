use sqlx::{
    Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{str::FromStr, time::Duration};
use thiserror::Error;

pub use sqlx::sqlite::{SqliteJournalMode, SqliteSynchronous};

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub busy_timeout: Duration,
    pub journal_mode: SqliteJournalMode,
    pub synchronous: SqliteSynchronous,
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection Error: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("Transaction error: {0}")]
    TransactionError(String),
}

impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let mut options = SqliteConnectOptions::from_str(&config.url)?
            .busy_timeout(config.busy_timeout)
            .journal_mode(config.journal_mode)
            .synchronous(config.synchronous)
            .create_if_missing(true);

        if config.url.starts_with("sqlite:memory") {
            options = options.shared_cache(true)
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(config.connect_timeout)
            .connect_with(options)
            .await?;

        pool.acquire()
            .await
            .map_err(|_| DatabaseError::ConnectionTimeout)?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn execute<'a, T>(
        &'a self,
        query: &'a str,
        params: &'a [&'a T],
    ) -> Result<u64, DatabaseError>
    where
        T: sqlx::Type<Sqlite> + sqlx::Encode<'a, Sqlite> + Send + Sync,
    {
        let query = params
            .iter()
            .fold(sqlx::query(query), |q, param| q.bind(*param));

        let result = query.execute(self.pool()).await?;
        Ok(result.rows_affected())
    }

    pub async fn fetch_one<'a, T, P>(
        &'a self,
        query: &'a str,
        params: &'a [&'a P],
    ) -> Result<T, DatabaseError>
    where
        T: Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
        P: sqlx::Type<Sqlite> + sqlx::Encode<'a, Sqlite> + Send + Sync,
    {
        let query = params
            .iter()
            .fold(sqlx::query(query), |q, param| q.bind(*param));

        let row = query.fetch_one(self.pool()).await?;

        Ok(T::from_row(&row)?)
    }
}
