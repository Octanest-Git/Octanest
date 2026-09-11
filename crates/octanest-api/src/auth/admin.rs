//! Admin auth settings RPC (`admin.auth.*`) — D-09, T-04-21/22.

use std::sync::Arc;

use octanest_core::{
    AppError, AuthSettingsPublic, EmailProviderKind, ProviderMode, UpdateAuthSettingsRequest,
};
use octanest_db::AuthSettingsRow;

use crate::email::{self, EmailSender};
use crate::rpc::RpcCtx;

fn env_nonempty(key: &str) -> bool {
    std::env::var(key)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

fn parse_provider_mode(s: &str) -> Result<ProviderMode, AppError> {
    match s {
        "local" => Ok(ProviderMode::Local),
        "workos" => Ok(ProviderMode::Workos),
        "oidc" => Ok(ProviderMode::Oidc),
        other => Err(AppError::new(
            "admin.invalid_provider_mode",
            format!("invalid provider_mode: {other}"),
        )),
    }
}

fn parse_email_provider(s: &str) -> Result<EmailProviderKind, AppError> {
    match s {
        "log" => Ok(EmailProviderKind::Log),
        "smtp" => Ok(EmailProviderKind::Smtp),
        "resend" => Ok(EmailProviderKind::Resend),
        other => Err(AppError::new(
            "admin.invalid_email_provider",
            format!("invalid email_provider: {other}"),
        )),
    }
}

fn mode_str(m: ProviderMode) -> &'static str {
    match m {
        ProviderMode::Local => "local",
        ProviderMode::Workos => "workos",
        ProviderMode::Oidc => "oidc",
    }
}

fn email_str(e: EmailProviderKind) -> &'static str {
    match e {
        EmailProviderKind::Log => "log",
        EmailProviderKind::Smtp => "smtp",
        EmailProviderKind::Resend => "resend",
    }
}

fn db_err(e: String) -> AppError {
    if e == "database not configured" {
        AppError::new(
            "db.not_configured",
            "no database configured for this instance",
        )
    } else {
        tracing::error!("admin auth settings db error: {e}");
        AppError::new("admin.internal", "auth settings operation failed")
    }
}

async fn require_admin(ctx: &RpcCtx) -> Result<(), AppError> {
    let Some(session) = &ctx.session else {
        return Err(AppError::new(
            "auth.unauthenticated",
            "not authenticated",
        ));
    };
    let user = ctx
        .db
        .find_user_by_id(&session.user_id)
        .await
        .map_err(db_err)?
        .ok_or_else(|| AppError::new("auth.unauthenticated", "not authenticated"))?;
    if !user.role.is_sys_admin() {
        return Err(AppError::new(
            "admin.forbidden",
            "You need system admin access to manage auth settings.",
        ));
    }
    Ok(())
}

/// Map DB row + ENV presence to public DTO (never includes secret values).
pub fn settings_to_public(row: &AuthSettingsRow) -> Result<AuthSettingsPublic, AppError> {
    let workos_client_id = row
        .workos_client_id
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("WORKOS_CLIENT_ID").ok().filter(|s| !s.is_empty()));

    let oidc_issuer = row
        .oidc_issuer
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var("OCTANEST_OIDC_ISSUER")
                .ok()
                .filter(|s| !s.is_empty())
        });

    let oidc_client_id = row
        .oidc_client_id
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var("OCTANEST_OIDC_CLIENT_ID")
                .ok()
                .filter(|s| !s.is_empty())
        });

    Ok(AuthSettingsPublic {
        provider_mode: parse_provider_mode(&row.provider_mode)?,
        email_provider: parse_email_provider(&row.email_provider)?,
        from_address: row.from_address.clone(),
        workos_client_id,
        oidc_issuer,
        oidc_client_id,
        smtp_configured: env_nonempty("OCTANEST_SMTP_URL"),
        resend_configured: env_nonempty("OCTANEST_RESEND_API_KEY"),
        workos_api_key_configured: env_nonempty("WORKOS_API_KEY"),
        oidc_client_secret_configured: env_nonempty("OCTANEST_OIDC_CLIENT_SECRET"),
        allow_signup: row.allow_signup,
    })
}

pub async fn get_settings(ctx: &RpcCtx) -> Result<AuthSettingsPublic, AppError> {
    require_admin(ctx).await?;
    let row = ctx.db.get_auth_settings().await.map_err(db_err)?;
    settings_to_public(&row)
}

pub async fn update_settings(
    ctx: &mut RpcCtx,
    input: serde_json::Value,
) -> Result<AuthSettingsPublic, AppError> {
    require_admin(ctx).await?;
    let req: UpdateAuthSettingsRequest = serde_json::from_value(input).map_err(|e| {
        AppError::new(
            "rpc.bad_input",
            format!("invalid update_settings input: {e}"),
        )
    })?;

    // Persist allow_signup from admin update request (06-01 DTOs).
    let row = ctx
        .db
        .update_auth_settings(
            mode_str(req.provider_mode),
            email_str(req.email_provider),
            req.from_address.as_deref(),
            req.oidc_issuer.as_deref(),
            req.oidc_client_id.as_deref(),
            req.workos_client_id.as_deref(),
            req.allow_signup,
        )
        .await
        .map_err(db_err)?;

    // Rebuild email sender from new settings + ENV secrets (never stored in DB).
    let rebuilt: Arc<dyn EmailSender> = email::build_email_sender_for_settings(&row);
    if let Ok(mut slot) = ctx.email_slot.write() {
        *slot = rebuilt.clone();
    }
    ctx.email = rebuilt;

    settings_to_public(&row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_settings_exposes_client_id_not_secrets() {
        let row = AuthSettingsRow {
            provider_mode: "local".into(),
            email_provider: "log".into(),
            from_address: Some("Octanest <noreply@test>".into()),
            oidc_issuer: None,
            oidc_client_id: None,
            workos_client_id: Some("client_abc".into()),
            allow_signup: false,
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let pub_ = settings_to_public(&row).expect("map");
        assert_eq!(pub_.provider_mode, ProviderMode::Local);
        assert_eq!(pub_.workos_client_id.as_deref(), Some("client_abc"));
        let json = serde_json::to_string(&pub_).unwrap();
        assert!(json.contains("workos_api_key_configured"));
        assert!(json.contains("smtp_configured"));
        assert!(json.contains("oidc_client_secret_configured"));
        // Boolean badge only — no actual secret value fields.
        assert!(!json.contains("\"api_key\":"));
        assert!(!json.contains("\"client_secret\":"));
        assert!(!json.contains("\"password\":"));
    }
}
