//! Thin HTTP routes mounted alongside RPC (auth callbacks, avatar, repo raw, Smart HTTP, LFS).

pub mod auth_callbacks;
pub mod avatar;
pub mod git_lfs;
pub mod git_smart_http;
pub mod release_assets;
pub mod repo_raw;
