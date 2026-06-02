use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ShortUrlRecord {
    pub code: String,
    pub long_url: String,
    pub access_count: i64,
}

#[derive(Debug, Clone)]
pub struct CreateShortUrlRecord {
    pub code: String,
    pub long_url: String,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("short code already exists")]
    DuplicateCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait ShortUrlRepository: Send + Sync {
    async fn create(&self, record: CreateShortUrlRecord)
    -> Result<ShortUrlRecord, RepositoryError>;

    async fn find_by_code(&self, code: &str) -> Result<Option<ShortUrlRecord>, RepositoryError>;

    async fn increment_access_count(&self, code: &str, amount: u64) -> Result<(), RepositoryError>;
}
