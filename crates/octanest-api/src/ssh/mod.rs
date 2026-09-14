//! Git-over-SSH (russh) — in-process listener gated by `OCTANEST_SSH_ENABLED`.

pub mod auth;
pub mod host_keys;
pub mod pack;
pub mod rate_limit;
pub mod server;

pub use server::{maybe_spawn_from_env, spawn_listener, ssh_enabled_from_env, SshState};
