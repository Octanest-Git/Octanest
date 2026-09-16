//! WebhookDispatcher — fan-out seam for domain events (D-HOOK-23).

use octanest_db::Database;
use serde_json::json;
use uuid::Uuid;

use super::deliver;

/// Emit a repository event to all active subscribed webhooks (best-effort async).
pub async fn emit(
    db: &Database,
    repository_id: &str,
    event: &str,
    action: &str,
    payload: serde_json::Value,
    env_name: &str,
) {
    let hooks = match db
        .list_active_webhooks_for_event(repository_id, event)
        .await
    {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(error = %e, "webhook emit: list hooks failed");
            return;
        }
    };
    if hooks.is_empty() {
        return;
    }

    let payload_json = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "webhook emit: serialize payload failed");
            return;
        }
    };

    for hook in hooks {
        let delivery_id = Uuid::new_v4().to_string();
        let delivery_guid = Uuid::new_v4().to_string();
        match db
            .insert_webhook_delivery(
                &delivery_id,
                &hook.id,
                &delivery_guid,
                event,
                action,
                &payload_json,
            )
            .await
        {
            Ok(_) => {
                deliver::spawn_deliver(
                    db.clone(),
                    hook.id.clone(),
                    delivery_id,
                    env_name.to_string(),
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, webhook_id = %hook.id, "webhook emit: insert delivery failed");
            }
        }
    }
}

/// Build a GitHub-shaped issues payload (D-HOOK-07 / D-HOOK-08).
pub fn issues_payload(
    action: &str,
    issue_number: i64,
    title: &str,
    body: &str,
    state: &str,
    owner: &str,
    repo_name: &str,
    repo_id: &str,
    sender_login: &str,
    sender_id: &str,
) -> serde_json::Value {
    json!({
        "action": action,
        "issue": {
            "number": issue_number,
            "title": title,
            "body": body,
            "state": state,
            "html_url": format!("/{owner}/{repo_name}/issues/{issue_number}"),
        },
        "repository": {
            "id": repo_id,
            "name": repo_name,
            "full_name": format!("{owner}/{repo_name}"),
            "owner": { "login": owner },
        },
        "sender": {
            "login": sender_login,
            "id": sender_id,
        }
    })
}

/// Synthetic ping payload (D-HOOK-05 / D-HOOK-21).
pub fn ping_payload(hook_id: &str, owner: &str, repo_name: &str) -> serde_json::Value {
    json!({
        "zen": "Octanest webhooks are ready.",
        "hook_id": hook_id,
        "repository": {
            "full_name": format!("{owner}/{repo_name}"),
        }
    })
}
