mod errors;

use errors::DatabaseError;
use sqlx::{
    PgPool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{str::FromStr, time::Duration};

pub use sqlx::types::{Json, JsonRawValue, JsonValue, Text, Type};

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let options = PgConnectOptions::from_str(&config.url)?;

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(config.connect_timeout)
            .connect_with(options)
            .await?;

        pool.acquire()
            .await
            .map_err(|_| DatabaseError::ConnectionTimeout)?;

        Ok(Self { pool })
    }

    /// Returns a reference to the `PgPool`.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn execute<'a, T>(
        &'a self,
        query: &'a str,
        params: &'a [&'a T],
    ) -> Result<u64, DatabaseError>
    where
        T: sqlx::Type<Postgres> + sqlx::Encode<'a, Postgres> + Send + Sync,
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
        T: Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>,
        P: sqlx::Type<Postgres> + sqlx::Encode<'a, Postgres> + Send + Sync,
    {
        let query = params
            .iter()
            .fold(sqlx::query(query), |q, param| q.bind(*param));

        let row = query.fetch_one(self.pool()).await?;
        Ok(T::from_row(&row)?)
    }

    pub async fn fetch_all<'a, T, P>(
        &'a self,
        query: &'a str,
        params: &'a [&'a P],
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>,
        P: sqlx::Type<Postgres> + sqlx::Encode<'a, Postgres> + Send + Sync,
    {
        let query = params
            .iter()
            .fold(sqlx::query(query), |q, param| q.bind(*param));

        let rows = query.fetch_all(self.pool()).await?;
        Ok(rows
            .iter()
            .map(|row| T::from_row(row))
            .collect::<Result<_, _>>()?)
    }

    /// Fetches all rows from the database.
    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, Postgres>, DatabaseError> {
        let tx = self.pool.begin().await?;
        Ok(tx)
    }

    /// Checks if the database connection is alive.
    pub async fn check_connection(&self) -> Result<bool, DatabaseError> {
        let result = sqlx::query("SELECT 1")
            .fetch_one(self.pool())
            .await
            .map(|_| ())
            .map_err(|_| DatabaseError::ConnectionError);

        Ok(result.is_ok())
    }

    /// Returns the number of connections in the `pool`.
    pub fn pool_stats(&self) -> u32 {
        self.pool.size()
    }
}
