use std::sync::Arc;

use thiserror::Error;

use crate::{
    domain::{short_code::validate_custom_code, url::validate_long_url},
    ports::{
        code_generator::CodeGenerator,
        short_url_repository::{
            CreateShortUrlRecord, RepositoryError, ShortUrlRecord, ShortUrlRepository,
        },
        url_cache::UrlCache,
    },
};

const MAX_GENERATION_ATTEMPTS: usize = 8;

#[derive(Debug, Clone)]
pub struct CreateShortUrlInput {
    pub long_url: String,
    pub custom_code: Option<String>,
}

#[derive(Debug, Error)]
pub enum CreateShortUrlError {
    #[error("invalid long url")]
    InvalidLongUrl,
    #[error("invalid custom code: {0}")]
    InvalidCustomCode(String),
    #[error("short code already exists")]
    DuplicateCode,
    #[error("could not generate a unique short code")]
    CodeGenerationExhausted,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct CreateShortUrlService {
    repository: Arc<dyn ShortUrlRepository>,
    cache: Arc<dyn UrlCache>,
    code_generator: Arc<dyn CodeGenerator>,
}

impl CreateShortUrlService {
    pub fn new(
        repository: Arc<dyn ShortUrlRepository>,
        cache: Arc<dyn UrlCache>,
        code_generator: Arc<dyn CodeGenerator>,
    ) -> Self {
        Self {
            repository,
            cache,
            code_generator,
        }
    }

    pub async fn execute(
        &self,
        input: CreateShortUrlInput,
    ) -> Result<ShortUrlRecord, CreateShortUrlError> {
        validate_long_url(&input.long_url).map_err(|_| CreateShortUrlError::InvalidLongUrl)?;

        if let Some(custom_code) = input.custom_code {
            validate_custom_code(&custom_code)
                .map_err(|error| CreateShortUrlError::InvalidCustomCode(error.to_string()))?;
            return self.create_once(custom_code, input.long_url).await;
        }

        for _ in 0..MAX_GENERATION_ATTEMPTS {
            let code = self.code_generator.generate().await;
            match self.create_once(code, input.long_url.clone()).await {
                Ok(record) => return Ok(record),
                Err(CreateShortUrlError::DuplicateCode) => continue,
                Err(error) => return Err(error),
            }
        }

        Err(CreateShortUrlError::CodeGenerationExhausted)
    }

    async fn create_once(
        &self,
        code: String,
        long_url: String,
    ) -> Result<ShortUrlRecord, CreateShortUrlError> {
        let record = self
            .repository
            .create(CreateShortUrlRecord {
                code,
                long_url: long_url.clone(),
            })
            .await
            .map_err(|error| match error {
                RepositoryError::DuplicateCode => CreateShortUrlError::DuplicateCode,
                RepositoryError::Other(error) => CreateShortUrlError::Internal(error),
            })?;

        self.cache
            .put_long_url(&record.code, &record.long_url)
            .await
            .map_err(CreateShortUrlError::Internal)?;

        Ok(record)
    }
}
