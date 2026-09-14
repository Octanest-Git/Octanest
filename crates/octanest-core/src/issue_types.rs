//! Issue domain DTOs and enums for later `issue.*` / `label.*` RPC (Phase 11).
//! No handlers here — schema door + shared types only (11-02).

use serde::{Deserialize, Serialize};

/// Issue lifecycle state (D-ISS-02): open ↔ closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

impl IssueState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            other => Err(format!("invalid issue state: {other}")),
        }
    }
}

/// GitHub eight reaction contents (D-ISS-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReactionContent {
    #[serde(rename = "+1")]
    PlusOne,
    #[serde(rename = "-1")]
    MinusOne,
    #[serde(rename = "laugh")]
    Laugh,
    #[serde(rename = "confused")]
    Confused,
    #[serde(rename = "heart")]
    Heart,
    #[serde(rename = "hooray")]
    Hooray,
    #[serde(rename = "rocket")]
    Rocket,
    #[serde(rename = "eyes")]
    Eyes,
}

impl ReactionContent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlusOne => "+1",
            Self::MinusOne => "-1",
            Self::Laugh => "laugh",
            Self::Confused => "confused",
            Self::Heart => "heart",
            Self::Hooray => "hooray",
            Self::Rocket => "rocket",
            Self::Eyes => "eyes",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "+1" => Ok(Self::PlusOne),
            "-1" => Ok(Self::MinusOne),
            "laugh" => Ok(Self::Laugh),
            "confused" => Ok(Self::Confused),
            "heart" => Ok(Self::Heart),
            "hooray" => Ok(Self::Hooray),
            "rocket" => Ok(Self::Rocket),
            "eyes" => Ok(Self::Eyes),
            other => Err(format!("invalid reaction content: {other}")),
        }
    }
}

/// Label definition scope (D-ISS-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LabelScope {
    Org,
    Repo,
}

/// Linked issue / PR stub kind (D-ISS-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueLinkKind {
    Issue,
    PrStub,
}

impl IssueLinkKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Issue => "issue",
            Self::PrStub => "pr_stub",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "issue" => Ok(Self::Issue),
            "pr_stub" => Ok(Self::PrStub),
            other => Err(format!("invalid issue link kind: {other}")),
        }
    }
}

/// Public issue metadata (list/detail).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuePublic {
    pub id: String,
    pub repo_id: String,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: IssueState,
    pub author_id: String,
    pub author_username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub labels: Vec<LabelPublic>,
    #[serde(default)]
    pub assignees: Vec<IssueAssigneePublic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAssigneePublic {
    pub user_id: String,
    pub username: String,
    #[serde(default)]
    pub display_name: String,
}

/// Label definition returned over RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelPublic {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub scope: LabelScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<String>,
    /// True when this org label is hidden for the current repo effective set.
    #[serde(default)]
    pub hidden: bool,
}

/// Create-issue input (RPC wired in 11-03).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub owner: String,
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
}

/// Issue get / mutate path — owner + repo name + `#N`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRefRequest {
    pub owner: String,
    pub name: String,
    pub number: i64,
}

/// List filters (D-ISS-16..18).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueListRequest {
    pub owner: String,
    pub name: String,
    /// `open` | `closed` | `all` — default open.
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueListResponse {
    pub issues: Vec<IssuePublic>,
    pub total: i64,
}

/// Comment on an issue (ISS-02).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCommentPublic {
    pub id: String,
    pub issue_id: String,
    pub author_id: String,
    pub author_username: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Linked issue / PR stub row (D-ISS-13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLinkPublic {
    pub id: String,
    pub kind: IssueLinkKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_repo_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_opaque_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_state_roundtrip() {
        assert_eq!(IssueState::parse("open").unwrap(), IssueState::Open);
        assert_eq!(IssueState::Closed.as_str(), "closed");
    }

    #[test]
    fn reaction_content_github_eight() {
        assert_eq!(ReactionContent::parse("+1").unwrap().as_str(), "+1");
        assert_eq!(ReactionContent::parse("eyes").unwrap().as_str(), "eyes");
        assert!(ReactionContent::parse("thumbs").is_err());
    }

    #[test]
    fn link_kind_pr_stub() {
        assert_eq!(
            IssueLinkKind::parse("pr_stub").unwrap(),
            IssueLinkKind::PrStub
        );
    }
}
