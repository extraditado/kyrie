mod errors;

use std::time::Duration;

use errors::CacheError;
use r2d2_redis::{RedisConnectionManager, r2d2, redis};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct RedisCache {
    pool: r2d2::Pool<RedisConnectionManager>,
    default_ttl: Duration,
    operation_timeout: Duration,
}

impl RedisCache {
    pub fn new(
        redis_url: &str,
        max_connections: u32,
        default_ttl: Duration,
        operation_timeout: Duration,
    ) -> Result<Self, CacheError> {
        let manager = RedisConnectionManager::new(redis_url)?;

        let pool = r2d2::Pool::builder()
            .max_size(max_connections)
            .test_on_check_out(true)
            .build(manager)?;

        Ok(Self {
            pool,
            default_ttl,
            operation_timeout,
        })
    }

    pub async fn get<K, V>(&self, key: &K) -> Result<Option<V>, CacheError>
    where
        K: Serialize + Sync + ?Sized,
        V: DeserializeOwned,
    {
        let serialized_key = rmp_serde::to_vec(key)?;
        let mut conn = self.get_connection().await?;

        let result: Option<Vec<u8>> = tokio::time::timeout(
            self.operation_timeout,
            tokio::task::spawn_blocking(move || {
                redis::cmd("GET").arg(serialized_key).query(&mut *conn)
            }),
        )
        .await
        .map_err(|_| CacheError::Timeout)?
        .map_err(|_| CacheError::Timeout)?
        .map_err(CacheError::from)?;

        match result {
            Some(data) => Ok(Some(rmp_serde::from_slice(&data)?)),
            None => Ok(None),
        }
    }

    pub async fn set<K, V>(&self, key: &K, value: V) -> Result<(), CacheError>
    where
        K: Serialize + Sync + ?Sized,
        V: Serialize + Sync,
    {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    pub async fn set_with_ttl<K, V>(
        &self,
        key: &K,
        value: V,
        ttl: Duration,
    ) -> Result<(), CacheError>
    where
        K: Serialize + Sync + ?Sized,
        V: Serialize + Sync,
    {
        let serialized_key = rmp_serde::to_vec(key)?;
        let serialized_value = rmp_serde::to_vec(&value)?;
        let mut conn = self.get_connection().await?;

        tokio::time::timeout(
            self.operation_timeout,
            tokio::task::spawn_blocking(move || {
                redis::cmd("SET")
                    .arg(serialized_key.as_slice())
                    .arg(serialized_value.as_slice())
                    .arg("PX")
                    .arg(ttl.as_millis() as u64)
                    .query::<()>(&mut *conn)
            }),
        )
        .await
        .map_err(|_| CacheError::Timeout)?
        .map_err(|_| CacheError::Timeout)?
        .map_err(CacheError::from)?;

        Ok(())
    }

    pub async fn delete<K: Serialize + ?Sized>(&self, key: &K) -> Result<(), CacheError> {
        let serialized_key = rmp_serde::to_vec(key)?;
        let mut conn = self.get_connection().await?;

        tokio::time::timeout(
            self.operation_timeout,
            tokio::task::spawn_blocking(move || {
                redis::cmd("DEL")
                    .arg(serialized_key)
                    .query::<()>(&mut *conn)
            }),
        )
        .await
        .map_err(|_| CacheError::Timeout)?
        .map_err(|_| CacheError::Timeout)?
        .map_err(CacheError::from)?;

        Ok(())
    }

    async fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<RedisConnectionManager>, CacheError> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || pool.get())
            .await
            .map_err(|_| CacheError::Timeout)?
            .map_err(Into::into)
    }
}
