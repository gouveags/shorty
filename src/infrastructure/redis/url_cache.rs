use async_trait::async_trait;
use redis::AsyncCommands;

use crate::ports::url_cache::UrlCache;

#[derive(Clone)]
pub struct RedisUrlCache {
    client: redis::Client,
    url_ttl_seconds: u64,
}

impl RedisUrlCache {
    pub fn new(redis_url: &str, url_ttl_seconds: u64) -> redis::RedisResult<Self> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
            url_ttl_seconds,
        })
    }

    fn url_key(code: &str) -> String {
        format!("shorty:url:{code}")
    }

    fn access_key(code: &str) -> String {
        format!("shorty:access_delta:{code}")
    }

    fn pending_access_codes_key() -> &'static str {
        "shorty:pending_access_codes"
    }
}

#[async_trait]
impl UrlCache for RedisUrlCache {
    async fn get_long_url(&self, code: &str) -> anyhow::Result<Option<String>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection.get(Self::url_key(code)).await?)
    }

    async fn put_long_url(&self, code: &str, long_url: &str) -> anyhow::Result<()> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let _: () = connection
            .set_ex(Self::url_key(code), long_url, self.url_ttl_seconds)
            .await?;
        Ok(())
    }

    async fn increment_accesses(&self, code: &str, amount: u64) -> anyhow::Result<()> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;

        let mut pipe = redis::pipe();
        pipe.atomic()
            .incr(Self::access_key(code), amount)
            .sadd(Self::pending_access_codes_key(), code);

        let _: () = pipe.query_async(&mut connection).await?;
        Ok(())
    }

    async fn pending_access_codes(&self) -> anyhow::Result<Vec<String>> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(connection
            .smembers(Self::pending_access_codes_key())
            .await?)
    }

    async fn take_access_count_delta(&self, code: &str) -> anyhow::Result<u64> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let script = redis::Script::new(
            r#"
            local value = redis.call("GET", KEYS[1])
            if value then
                redis.call("DEL", KEYS[1])
                return tonumber(value)
            end
            return 0
            "#,
        );

        Ok(script
            .key(Self::access_key(code))
            .invoke_async(&mut connection)
            .await?)
    }
}
