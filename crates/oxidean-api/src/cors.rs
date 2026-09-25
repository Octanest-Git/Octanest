use axum::http::{header, HeaderName, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

fn allowed_headers() -> [HeaderName; 4] {
    [
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::ACCEPT,
        HeaderName::from_static("oxidean-rpc-version"),
    ]
}

pub fn build_cors(env_name: &str, origins: Option<&str>) -> Result<CorsLayer, String> {
    let is_dev = env_name == "development" || env_name == "dev";

    if is_dev {
        return Ok(CorsLayer::new()
            .allow_credentials(true)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(allowed_headers())
            .allow_origin(AllowOrigin::mirror_request()));
    }

    let list = origins
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            "OXIDEAN_CORS_ORIGINS is required when OXIDEAN_ENV is not development".to_string()
        })?;

    let parsed: Result<Vec<HeaderValue>, _> = list
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|o| {
            o.parse::<HeaderValue>()
                .map_err(|e| format!("invalid CORS origin {o:?}: {e}"))
        })
        .collect();
    let parsed = parsed?;
    if parsed.is_empty() {
        return Err("OXIDEAN_CORS_ORIGINS must list at least one origin".into());
    }

    Ok(CorsLayer::new()
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(allowed_headers())
        .allow_origin(AllowOrigin::list(parsed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prod_requires_allowlist() {
        assert!(build_cors("production", None).is_err());
        assert!(build_cors("production", Some("http://localhost:3000")).is_ok());
    }

    #[test]
    fn dev_ok_without_list() {
        assert!(build_cors("development", None).is_ok());
    }
}
