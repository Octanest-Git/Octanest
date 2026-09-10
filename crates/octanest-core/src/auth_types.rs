//! Auth-related DTOs and username validation shared across API and clients.

use serde::{Deserialize, Serialize};

/// Auth provider mode (D-05, D-06). Serialized as lowercase: `local` | `workos` | `oidc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    Local,
    Workos,
    Oidc,
}

/// Outbound email adapter kind. Serialized as lowercase: `log` | `smtp` | `resend`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailProviderKind {
    Log,
    Smtp,
    Resend,
}

/// Public user profile returned over RPC (no password hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub bio: String,
    /// Public URL path (e.g. `/uploads/avatars/{id}.webp`), not a filesystem path.
    pub avatar_url: Option<String>,
    pub is_admin: bool,
    /// True when username needs completion (e.g. after SSO with placeholder handle).
    pub profile_incomplete: bool,
}

/// Local signup input (D-01).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

/// Local login input (D-02). `identifier` is email or username.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
    pub remember_me: bool,
}

/// Public provider config for `auth.provider_config`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigPublic {
    pub mode: ProviderMode,
}

/// Profile update fields (D-18). Bio max length enforced in API (160).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: String,
    pub username: String,
    pub bio: String,
}

/// Instance auth settings for admin UI — secrets never returned; ENV badges only (D-09, T-04-22).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettingsPublic {
    pub provider_mode: ProviderMode,
    pub email_provider: EmailProviderKind,
    pub from_address: Option<String>,
    /// Non-secret WorkOS client id (display); may also come from ENV when DB empty.
    pub workos_client_id: Option<String>,
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    pub smtp_configured: bool,
    pub resend_configured: bool,
    pub workos_api_key_configured: bool,
    pub oidc_client_secret_configured: bool,
}

/// Admin update payload — non-secret fields only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAuthSettingsRequest {
    pub provider_mode: ProviderMode,
    pub email_provider: EmailProviderKind,
    pub from_address: Option<String>,
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    pub workos_client_id: Option<String>,
}

const RESERVED_USERNAMES: &[&str] = &[
    "admin",
    "api",
    "settings",
    "login",
    "signup",
    "logout",
    "status",
    "dashboard",
    "explore",
    "orgs",
    "org",
    "help",
    "support",
    "www",
    "root",
    "system",
    "null",
    "undefined",
    "octanest",
    "assets",
    "static",
    "uploads",
    "health",
    "rpc",
    "auth",
    "account",
    "profile",
    "robots",
    "favicon",
];

/// Returns true if `u` matches a reserved username (case-insensitive).
pub fn is_reserved_username(u: &str) -> bool {
    let lower = u.to_ascii_lowercase();
    RESERVED_USERNAMES.iter().any(|r| *r == lower.as_str())
}

/// GitHub-like username rules (D-03): 1–39 chars, ascii alphanumeric + hyphen,
/// no leading/trailing hyphen, not reserved.
pub fn validate_username(raw: &str) -> Result<(), String> {
    let u = raw.trim();
    if u.is_empty() || u.len() > 39 {
        return Err("username must be 1–39 characters".into());
    }
    if u.starts_with('-') || u.ends_with('-') {
        return Err("username cannot start or end with a hyphen".into());
    }
    if !u.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("username must be alphanumeric or hyphen".into());
    }
    if is_reserved_username(u) {
        return Err("username is reserved".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_username_accepts_short_ok() {
        assert!(validate_username("ab").is_ok());
    }

    #[test]
    fn validate_username_rejects_leading_hyphen() {
        assert!(validate_username("-ab").is_err());
    }

    #[test]
    fn validate_username_rejects_trailing_hyphen() {
        assert!(validate_username("ab-").is_err());
    }

    #[test]
    fn validate_username_rejects_reserved() {
        let err = validate_username("admin").unwrap_err();
        assert!(err.contains("reserved"));
    }

    #[test]
    fn validate_username_rejects_too_long() {
        let long = "a".repeat(40);
        assert!(validate_username(&long).is_err());
    }

    #[test]
    fn provider_mode_serde_lowercase() {
        let json = serde_json::to_string(&ProviderMode::Local).unwrap();
        assert_eq!(json, "\"local\"");
        let mode: ProviderMode = serde_json::from_str("\"workos\"").unwrap();
        assert_eq!(mode, ProviderMode::Workos);
    }
}
