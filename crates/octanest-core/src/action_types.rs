//! Actions run/job RPC types (Phase 19 / ACT-03).

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
