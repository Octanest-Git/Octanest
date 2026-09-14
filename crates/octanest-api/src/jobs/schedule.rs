//! Interval timers for orphan reconcile + git gc (D-36 / D-37).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use octanest_db::Database;
use octanest_git::GitBackend;
use tokio::time::{interval, MissedTickBehavior};

use super::reconcile::{orphan_reconcile, run_gc_all};

/// Background job intervals from ENV (0 disables that job).
#[derive(Debug, Clone, Copy)]
pub struct JobConfig {
    /// Default 24h. Set `OCTANEST_ORPHAN_RECONCILE_INTERVAL_SECS=0` to disable.
    pub orphan_interval: Duration,
    /// Default 7d. Set `OCTANEST_GIT_GC_INTERVAL_SECS=0` to disable.
    pub gc_interval: Duration,
}

impl JobConfig {
    pub fn from_env() -> Self {
        let orphan_secs = parse_u64_env("OCTANEST_ORPHAN_RECONCILE_INTERVAL_SECS", 86_400);
        let gc_secs = parse_u64_env("OCTANEST_GIT_GC_INTERVAL_SECS", 604_800);
        Self {
            orphan_interval: Duration::from_secs(orphan_secs),
            gc_interval: Duration::from_secs(gc_secs),
        }
    }
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Spawn in-process orphan reconcile + scheduled gc loops (A3 RESEARCH).
pub fn spawn_background_jobs(
    db: Database,
    git: Arc<dyn GitBackend>,
    repos_dir: PathBuf,
    config: JobConfig,
) {
    if config.orphan_interval.as_secs() > 0 {
        let db_o = db.clone();
        let repos_o = repos_dir.clone();
        let period = config.orphan_interval;
        tokio::spawn(async move {
            let mut ticker = interval(period);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            // First tick completes immediately — run once after boot, then on interval.
            loop {
                ticker.tick().await;
                match orphan_reconcile(&db_o, &repos_o).await {
                    Ok(stats) => {
                        if stats.orphans_removed > 0
                            || stats.purged_soft_deleted > 0
                            || stats.purged_expired_redirects > 0
                            || stats.errors > 0
                        {
                            tracing::info!(
                                orphans = stats.orphans_removed,
                                purged = stats.purged_soft_deleted,
                                redirects = stats.purged_expired_redirects,
                                errors = stats.errors,
                                "orphan reconcile completed"
                            );
                        } else {
                            tracing::debug!("orphan reconcile: nothing to do");
                        }
                    }
                    Err(e) => tracing::error!(error = %e, "orphan reconcile failed"),
                }
            }
        });
        tracing::info!(
            secs = config.orphan_interval.as_secs(),
            "orphan reconcile job scheduled"
        );
    } else {
        tracing::info!("orphan reconcile job disabled (interval 0)");
    }

    if config.gc_interval.as_secs() > 0 {
        let db_g = db;
        let git_g = git;
        let repos_g = repos_dir;
        let period = config.gc_interval;
        tokio::spawn(async move {
            let mut ticker = interval(period);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                match run_gc_all(&db_g, git_g.as_ref(), &repos_g).await {
                    Ok((ok, err)) => {
                        if ok > 0 || err > 0 {
                            tracing::info!(gc_ok = ok, gc_err = err, "scheduled git gc completed");
                        } else {
                            tracing::debug!("scheduled git gc: no active repos");
                        }
                    }
                    Err(e) => tracing::error!(error = %e, "scheduled git gc failed"),
                }
            }
        });
        tracing::info!(
            secs = config.gc_interval.as_secs(),
            "git gc job scheduled"
        );
    } else {
        tracing::info!("git gc job disabled (interval 0)");
    }
}
