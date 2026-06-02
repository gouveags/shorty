use async_trait::async_trait;

#[async_trait]
pub trait UrlCache: Send + Sync {
    async fn get_long_url(&self, code: &str) -> anyhow::Result<Option<String>>;
    async fn put_long_url(&self, code: &str, long_url: &str) -> anyhow::Result<()>;
    async fn increment_accesses(&self, code: &str, amount: u64) -> anyhow::Result<()>;
    async fn pending_access_codes(&self) -> anyhow::Result<Vec<String>>;
    async fn take_access_count_delta(&self, code: &str) -> anyhow::Result<u64>;
}
