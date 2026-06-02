use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub custom_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUrlResponse {
    pub code: String,
    pub short_url: String,
    pub long_url: String,
}

#[derive(Debug, Serialize)]
pub struct UrlStatsResponse {
    pub code: String,
    pub long_url: String,
    pub access_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
