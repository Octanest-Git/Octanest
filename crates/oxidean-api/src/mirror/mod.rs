//! Two-way repository mirroring (GIT-V2-01).

mod engine;
mod hook;
mod queue;
mod rpc;

pub use engine::run_mirror_sync;
pub use hook::mirror_hook;
pub use queue::{enqueue_mirror_for_repo, notify_mirror_after_local_mutation, spawn_mirror_poller};
pub use rpc::{
    delete, fetch_host_key, generate_ssh_key, get, rotate_webhook_secret, sync_now, upsert,
};
