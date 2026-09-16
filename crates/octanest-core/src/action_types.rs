//! Actions run/job/secrets RPC types (Phase 19 / ACT-03 / ACT-06).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunPublic {
    pub id: String,
    pub repository_id: String,
    pub workflow_path: String,
    pub workflow_name: String,
    pub event: String,
    pub head_sha: String,
    pub head_ref: String,
    pub status: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionJobPublic {
    pub id: String,
    pub run_id: String,
    pub job_key: String,
    pub name: String,
    pub status: String,
    pub runs_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunsListRequest {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunsListResponse {
    pub runs: Vec<ActionRunPublic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunGetRequest {
    pub owner: String,
    pub name: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunGetResponse {
    pub run: ActionRunPublic,
    pub jobs: Vec<ActionJobPublic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionJobLogRequest {
    pub owner: String,
    pub name: String,
    pub run_id: String,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionJobLogResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSecretPublic {
    pub name: String,
    pub updated_at: String,
}

pub type ActionSecretMetaPublic = ActionSecretPublic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSecretsListRequest {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSecretsListResponse {
    pub secrets: Vec<ActionSecretPublic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSecretsPutRequest {
    pub owner: String,
    pub name: String,
    pub secret_name: String,
    pub value: String,
}

pub type ActionSecretPutRequest = ActionSecretsPutRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSecretsDeleteRequest {
    pub owner: String,
    pub name: String,
    pub secret_name: String,
}

pub type ActionSecretDeleteRequest = ActionSecretsDeleteRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoActionsEnabledRequest {
    pub owner: String,
    pub name: String,
}

pub type ActionEnabledRequest = RepoActionsEnabledRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoActionsSetEnabledRequest {
    pub owner: String,
    pub name: String,
    pub enabled: bool,
}

pub type ActionSetEnabledRequest = RepoActionsSetEnabledRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoActionsEnabledResponse {
    pub enabled: bool,
}

pub type ActionEnabledResponse = RepoActionsEnabledResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminActionsCreateRegistrationTokenResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRegistrationTokenResponse {
    pub token: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunnerPublic {
    pub id: String,
    pub name: String,
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<String>,
    pub ephemeral: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_online: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminActionsListRunnersResponse {
    pub runners: Vec<ActionRunnerPublic>,
}

pub type ActionListRunnersResponse = AdminActionsListRunnersResponse;
