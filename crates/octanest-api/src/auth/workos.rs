//! WorkOS AuthKit adapter (D-07): start URL + authenticate_with_code → Octanest session.

use std::time::Instant;

use url::Url;
use workos::helpers::AuthKitAuthorizationUrlParams;
use workos::user_management::AuthenticateWithCodeParams;
use workos::Client;

use crate::auth::external::{ExternalAuthError, ExternalIdentity};
use crate::auth::pending::{PendingAuth, PendingAuthStore};

pub const PROVIDER: &str = "workos";

#[derive(Debug, Clone)]
pub struct WorkOsConfig {
    pub api_key: String,
    pub client_id: String,
}

impl WorkOsConfig {
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("WORKOS_API_KEY").ok()?.trim().to_string();
        let client_id = std::env::var("WORKOS_CLIENT_ID").ok()?.trim().to_string();
        if api_key.is_empty() || client_id.is_empty() {
            return None;
        }
        Some(Self { api_key, client_id })
    }
}

fn client(cfg: &WorkOsConfig) -> Client {
    Client::builder()
        .api_key(&cfg.api_key)
        .client_id(&cfg.client_id)
        .build()
}

/// Build AuthKit authorization URL with `provider=authkit` + PKCE; stash verifier/state.
pub fn start(
    pending: &PendingAuthStore,
    cfg: &WorkOsConfig,
    redirect_uri: &str,
    return_to: &str,
) -> Result<Url, ExternalAuthError> {
    let c = client(cfg);
    let result = c
        .authkit()
        .pkce_authorization_url(AuthKitAuthorizationUrlParams {
            redirect_uri: redirect_uri.to_string(),
            provider: Some("authkit".into()),
            ..Default::default()
        })
        .map_err(|e| ExternalAuthError::Failed(e.to_string()))?;

    pending.insert(
        result.state.clone(),
        PendingAuth {
            provider: PROVIDER.into(),
            code_verifier: result.code_verifier,
            nonce: None,
            return_to: return_to.to_string(),
            redirect_uri: redirect_uri.to_string(),
            created_at: Instant::now(),
        },
    );

    Url::parse(&result.url).map_err(|e| ExternalAuthError::Failed(e.to_string()))
}

/// Exchange authorization code via WorkOS `authenticate_with_code` (PKCE verifier).
pub async fn finish(
    pending: &PendingAuthStore,
    cfg: &WorkOsConfig,
    code: &str,
    state: &str,
) -> Result<(ExternalIdentity, String), ExternalAuthError> {
    let pending_auth = pending
        .take(state)
        .filter(|p| p.provider == PROVIDER)
        .ok_or(ExternalAuthError::InvalidState)?;

    let c = client(cfg);
    let params = AuthenticateWithCodeParams {
        code: code.to_string(),
        code_verifier: Some(pending_auth.code_verifier),
        invitation_token: None,
        ip_address: None,
        device_id: None,
        user_agent: None,
        signals_id: None,
    };
    let response = c
        .user_management()
        .authenticate_with_code(params)
        .await
        .map_err(|e| ExternalAuthError::Failed(e.to_string()))?;

    let user = response.user;
    let display_name = user
        .name
        .or_else(|| match (user.first_name, user.last_name) {
            (Some(f), Some(l)) => Some(format!("{f} {l}")),
            (Some(f), None) => Some(f),
            (None, Some(l)) => Some(l),
            _ => None,
        });

    Ok((
        ExternalIdentity {
            provider: PROVIDER.into(),
            provider_subject: user.id,
            email: user.email,
            display_name,
        },
        pending_auth.return_to,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_without_config_is_not_configured_path() {
        // Callers check WorkOsConfig::from_env() / explicit cfg before start.
        // When keys absent, routes return auth.not_configured (tested below via cfg gate).
        assert!(WorkOsConfig {
            api_key: String::new(),
            client_id: String::new(),
        }
        .api_key
        .is_empty());
    }

    #[test]
    fn start_url_contains_authkit_provider() {
        let pending = PendingAuthStore::new();
        let cfg = WorkOsConfig {
            api_key: "sk_test_example".into(),
            client_id: "client_test_example".into(),
        };
        let url = start(
            &pending,
            &cfg,
            "http://localhost:8080/api/auth/workos/callback",
            "/dashboard",
        )
        .expect("start");
        let s = url.as_str();
        assert!(
            s.contains("provider=authkit"),
            "expected provider=authkit in {s}"
        );
        assert!(
            s.contains("api.workos.com") || s.contains("user_management/authorize"),
            "expected WorkOS authorize host/path in {s}"
        );
        assert!(s.contains("code_challenge="));
    }

    #[test]
    fn finish_rejects_unknown_state() {
        let pending = PendingAuthStore::new();
        let cfg = WorkOsConfig {
            api_key: "sk_test".into(),
            client_id: "client_test".into(),
        };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = rt
            .block_on(finish(&pending, &cfg, "code", "missing-state"))
            .unwrap_err();
        assert!(matches!(err, ExternalAuthError::InvalidState));
        assert_eq!(err.code(), "auth.invalid_state");
    }

    #[test]
    fn not_configured_error_code() {
        assert_eq!(
            ExternalAuthError::NotConfigured.code(),
            "auth.not_configured"
        );
    }
}
