//! Package registry RPC DTOs (PKG-05).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesListRequest {
    /// Owner username or org slug.
    #[serde(default)]
    pub owner: Option<String>,
    /// Filter to packages linked to this repository id.
    #[serde(default)]
    pub repository_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersionPublic {
    pub version: String,
    pub digest: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePublic {
    pub id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub name: String,
    pub format: String,
    pub visibility: String,
    pub repository_id: Option<String>,
    pub versions: Vec<PackageVersionPublic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesListResponse {
    pub packages: Vec<PackagePublic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesDeleteVersionRequest {
    pub package_id: String,
    pub version: String,
    /// Must equal `{name}@{version}` (D-PKG-12).
    pub confirm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesDeleteVersionResponse {
    pub ok: bool,
}
