use std::{sync::Arc, time::Duration};

use crate::ports::{short_url_repository::ShortUrlRepository, url_cache::UrlCache};

#[derive(Clone)]
pub struct AccessCountFlusher {
    repository: Arc<dyn ShortUrlRepository>,
    cache: Arc<dyn UrlCache>,
    interval: Duration,
}

impl AccessCountFlusher {
    pub fn new(
        repository: Arc<dyn ShortUrlRepository>,
        cache: Arc<dyn UrlCache>,
        interval: Duration,
    ) -> Self {
        Self {
            repository,
            cache,
            interval,
        }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;

            if let Err(error) = self.flush_once().await {
                tracing::error!(?error, "failed to flush access counts");
            }
        }
    }

    pub async fn flush_once(&self) -> anyhow::Result<()> {
        let codes = self.cache.pending_access_codes().await?;

        for code in codes {
            let delta = self.cache.take_access_count_delta(&code).await?;

            if delta > 0 {
                self.repository.increment_access_count(&code, delta).await?;
            }
        }

        Ok(())
    }
}
