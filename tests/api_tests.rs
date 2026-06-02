use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use shorty::{
    app::state::AppState,
    application::{
        create_short_url::CreateShortUrlService, get_short_url_stats::GetShortUrlStatsService,
        resolve_short_url::ResolveShortUrlService,
    },
    interfaces::http::routes::router,
    ports::{
        code_generator::CodeGenerator,
        short_url_repository::{
            CreateShortUrlRecord, RepositoryError, ShortUrlRecord, ShortUrlRepository,
        },
        url_cache::UrlCache,
    },
};
use tower::ServiceExt;

#[derive(Default)]
struct FakeRepository {
    records: Mutex<HashMap<String, ShortUrlRecord>>,
}

#[async_trait]
impl ShortUrlRepository for FakeRepository {
    async fn create(
        &self,
        record: CreateShortUrlRecord,
    ) -> Result<ShortUrlRecord, RepositoryError> {
        let mut records = self.records.lock().unwrap();

        if records.contains_key(&record.code) {
            return Err(RepositoryError::DuplicateCode);
        }

        let record = ShortUrlRecord {
            code: record.code,
            long_url: record.long_url,
            access_count: 0,
        };
        records.insert(record.code.clone(), record.clone());

        Ok(record)
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<ShortUrlRecord>, RepositoryError> {
        Ok(self.records.lock().unwrap().get(code).cloned())
    }

    async fn increment_access_count(&self, code: &str, amount: u64) -> Result<(), RepositoryError> {
        if let Some(record) = self.records.lock().unwrap().get_mut(code) {
            record.access_count += amount as i64;
        }

        Ok(())
    }
}

#[derive(Default)]
struct FakeCache {
    urls: Mutex<HashMap<String, String>>,
    deltas: Mutex<HashMap<String, u64>>,
    pending: Mutex<HashSet<String>>,
}

#[async_trait]
impl UrlCache for FakeCache {
    async fn get_long_url(&self, code: &str) -> anyhow::Result<Option<String>> {
        Ok(self.urls.lock().unwrap().get(code).cloned())
    }

    async fn put_long_url(&self, code: &str, long_url: &str) -> anyhow::Result<()> {
        self.urls
            .lock()
            .unwrap()
            .insert(code.to_string(), long_url.to_string());
        Ok(())
    }

    async fn increment_accesses(&self, code: &str, amount: u64) -> anyhow::Result<()> {
        *self
            .deltas
            .lock()
            .unwrap()
            .entry(code.to_string())
            .or_default() += amount;
        self.pending.lock().unwrap().insert(code.to_string());
        Ok(())
    }

    async fn pending_access_codes(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.pending.lock().unwrap().iter().cloned().collect())
    }

    async fn take_access_count_delta(&self, code: &str) -> anyhow::Result<u64> {
        Ok(self.deltas.lock().unwrap().remove(code).unwrap_or_default())
    }
}

struct StaticCodeGenerator;

#[async_trait]
impl CodeGenerator for StaticCodeGenerator {
    async fn generate(&self) -> String {
        "abc1234".to_string()
    }
}

fn app() -> axum::Router {
    let repository: Arc<dyn ShortUrlRepository> = Arc::new(FakeRepository::default());
    let cache: Arc<dyn UrlCache> = Arc::new(FakeCache::default());
    let code_generator: Arc<dyn CodeGenerator> = Arc::new(StaticCodeGenerator);

    let state = AppState {
        public_base_url: "http://localhost:8080".to_string(),
        create_short_url: CreateShortUrlService::new(
            Arc::clone(&repository),
            Arc::clone(&cache),
            code_generator,
        ),
        resolve_short_url: ResolveShortUrlService::new(Arc::clone(&repository), Arc::clone(&cache)),
        get_short_url_stats: GetShortUrlStatsService::new(repository, cache),
    };

    router(state)
}

#[tokio::test]
async fn creates_custom_url_and_redirects() {
    let app = app();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "url": "https://example.com/hello",
                        "custom_code": "hello"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let redirect_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(redirect_response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        redirect_response.headers().get("location").unwrap(),
        "https://example.com/hello"
    );

    let stats_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/urls/hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(stats_response.status(), StatusCode::OK);
    let body = stats_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let stats: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats["access_count"], 1);
}

#[tokio::test]
async fn rejects_duplicate_custom_url() {
    let app = app();

    for expected_status in [StatusCode::CREATED, StatusCode::CONFLICT] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/urls")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "url": "https://example.com/hello",
                            "custom_code": "hello"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), expected_status);
    }
}
