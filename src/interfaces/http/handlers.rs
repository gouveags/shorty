use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};

use crate::{
    app::state::AppState,
    application::create_short_url::CreateShortUrlInput,
    interfaces::http::{
        dto::{CreateUrlRequest, CreateUrlResponse, UrlStatsResponse},
        error::ApiError,
    },
};

pub async fn health() -> &'static str {
    "ok"
}

pub async fn create_url(
    State(state): State<AppState>,
    Json(request): Json<CreateUrlRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let record = state
        .create_short_url
        .execute(CreateShortUrlInput {
            long_url: request.url,
            custom_code: request.custom_code,
        })
        .await?;

    let short_url = format!(
        "{}/{}",
        state.public_base_url.trim_end_matches('/'),
        record.code
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateUrlResponse {
            code: record.code,
            short_url,
            long_url: record.long_url,
        }),
    ))
}

pub async fn redirect_to_url(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Redirect, ApiError> {
    let Some(long_url) = state
        .resolve_short_url
        .execute(&code)
        .await
        .map_err(ApiError::internal)?
    else {
        return Err(ApiError::not_found("short url not found"));
    };

    Ok(Redirect::temporary(&long_url))
}

pub async fn get_url_stats(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<UrlStatsResponse>, ApiError> {
    let Some(record) = state
        .get_short_url_stats
        .execute(&code)
        .await
        .map_err(ApiError::internal)?
    else {
        return Err(ApiError::not_found("short url not found"));
    };

    Ok(Json(UrlStatsResponse {
        code: record.code,
        long_url: record.long_url,
        access_count: record.access_count,
    }))
}
