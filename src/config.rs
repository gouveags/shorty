use std::{env, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub public_base_url: String,
    pub database_url: String,
    pub redis_url: String,
    pub code_length: usize,
    pub cache_ttl_seconds: u64,
    pub access_flush_interval: Duration,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            code_length: env::var("CODE_LENGTH")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(7),
            cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(86_400),
            access_flush_interval: Duration::from_secs(
                env::var("ACCESS_FLUSH_INTERVAL_SECONDS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(2),
            ),
        })
    }
}
