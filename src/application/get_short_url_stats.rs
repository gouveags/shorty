use std::sync::Arc;

use crate::ports::{
    short_url_repository::{ShortUrlRecord, ShortUrlRepository},
    url_cache::UrlCache,
};

#[derive(Clone)]
pub struct GetShortUrlStatsService {
    repository: Arc<dyn ShortUrlRepository>,
    cache: Arc<dyn UrlCache>,
}

impl GetShortUrlStatsService {
    pub fn new(repository: Arc<dyn ShortUrlRepository>, cache: Arc<dyn UrlCache>) -> Self {
        Self { repository, cache }
    }

    pub async fn execute(&self, code: &str) -> anyhow::Result<Option<ShortUrlRecord>> {
        let Some(mut record) = self.repository.find_by_code(code).await? else {
            return Ok(None);
        };

        if let Ok(delta) = self.cache.take_access_count_delta(code).await {
            if delta > 0 {
                self.repository.increment_access_count(code, delta).await?;
                record.access_count += delta as i64;
            }
        }

        Ok(Some(record))
    }
}
