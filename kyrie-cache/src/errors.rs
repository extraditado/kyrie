use r2d2_redis::r2d2;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    ConnectionError(#[from] r2d2::Error),

    #[error("Redis command error: {0}")]
    RedisError(#[from] r2d2_redis::redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),

    #[error("Operation timeout")]
    Timeout,
}
