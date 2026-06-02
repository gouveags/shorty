use std::sync::Arc;

use crate::ports::{short_url_repository::ShortUrlRepository, url_cache::UrlCache};

#[derive(Clone)]
pub struct ResolveShortUrlService {
    repository: Arc<dyn ShortUrlRepository>,
    cache: Arc<dyn UrlCache>,
}

impl ResolveShortUrlService {
    pub fn new(repository: Arc<dyn ShortUrlRepository>, cache: Arc<dyn UrlCache>) -> Self {
        Self { repository, cache }
    }

    pub async fn execute(&self, code: &str) -> anyhow::Result<Option<String>> {
        if let Some(long_url) = self.cache.get_long_url(code).await? {
            self.cache.increment_accesses(code, 1).await?;
            return Ok(Some(long_url));
        }

        let Some(record) = self.repository.find_by_code(code).await? else {
            return Ok(None);
        };

        self.cache.put_long_url(code, &record.long_url).await?;
        self.cache.increment_accesses(code, 1).await?;

        Ok(Some(record.long_url))
    }
}
