use redis::{AsyncCommands, Client, RedisError, ToSingleRedisArg};
use std::result::Result;

#[derive(Clone)]
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;

        Ok(Self { client })
    }

    pub async fn incr(&self, k: &str) -> Result<(), RedisError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.incr(k, 1).await
    }

    pub async fn set<T>(&self, k: String, v: T, exp: u64) -> Result<(), RedisError>
    where
        T: ToSingleRedisArg + Send + Sync,
    {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.set_ex(k, v, exp).await
    }

    pub async fn get(&self, k: &str) -> Result<Option<String>, RedisError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.get(k).await
    }

    pub async fn exists(&self, k: &str) -> Result<bool, RedisError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.exists(k).await
    }

    pub async fn revoke(&self, k: String) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.del(k).await
    }
}
