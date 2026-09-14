//! OCI Distribution Spec stubs + discovery (expanded in 20-05).

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use crate::app::AppState;

/// GET /v2 or /v2/ — registry API version discovery (Docker-Distribution-API-Version).
pub async fn discovery() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::HeaderName::from_static("docker-distribution-api-version"),
            HeaderValue::from_static("registry/2.0"),
        )],
    )
}

async fn not_implemented() -> StatusCode {
    StatusCode::NOT_FOUND
}

/// Nested under `/v2` for non-root paths only (root discovery is mounted on the app router).
pub fn router() -> Router<AppState> {
    Router::new().route("/{*rest}", get(not_implemented))
}
