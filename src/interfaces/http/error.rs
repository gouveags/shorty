use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    application::create_short_url::CreateShortUrlError, interfaces::http::dto::ErrorResponse,
};

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    pub fn internal(error: anyhow::Error) -> Self {
        tracing::error!(?error, "request failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal server error".to_string(),
        }
    }
}

impl From<CreateShortUrlError> for ApiError {
    fn from(error: CreateShortUrlError) -> Self {
        match error {
            CreateShortUrlError::InvalidLongUrl => Self {
                status: StatusCode::BAD_REQUEST,
                message: "invalid long url".to_string(),
            },
            CreateShortUrlError::InvalidCustomCode(message) => Self {
                status: StatusCode::BAD_REQUEST,
                message,
            },
            CreateShortUrlError::DuplicateCode => Self {
                status: StatusCode::CONFLICT,
                message: "short code already exists".to_string(),
            },
            CreateShortUrlError::CodeGenerationExhausted => Self {
                status: StatusCode::SERVICE_UNAVAILABLE,
                message: "could not generate a unique short code".to_string(),
            },
            CreateShortUrlError::Internal(error) => Self::internal(error),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}
