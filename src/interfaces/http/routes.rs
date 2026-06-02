use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

use crate::{app::state::AppState, interfaces::http::handlers};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/urls", post(handlers::create_url))
        .route("/urls/{code}", get(handlers::get_url_stats))
        .route("/{code}", get(handlers::redirect_to_url))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
