//! npm registry stubs (expanded in 20-06).

use axum::http::StatusCode;
use axum::routing::{any, get};
use axum::Router;

use crate::app::AppState;

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(not_found))
        .route("/{*rest}", any(not_found))
}
