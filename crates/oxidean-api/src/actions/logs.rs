//! Job log files under OXIDEAN_ACTIONS_LOG_DIR (D-ACT-13).

use std::path::{Path, PathBuf};

use tokio::fs;

/// `{log_dir}/{run_id}/{job_id}.log` — IDs must not contain path separators (T-19-05).
pub fn job_log_path(log_dir: &Path, run_id: &str, job_id: &str) -> Result<PathBuf, String> {
    reject_id(run_id)?;
    reject_id(job_id)?;
    Ok(log_dir.join(run_id).join(format!("{job_id}.log")))
}

fn reject_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err("invalid actions log id".into());
    }
    Ok(())
}

pub async fn append_job_log(
    log_dir: &Path,
    run_id: &str,
    job_id: &str,
    chunk: &[u8],
) -> Result<(), String> {
    let path = job_log_path(log_dir, run_id, job_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("create log dir: {e}"))?;
    }
    use tokio::io::AsyncWriteExt;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| format!("open log: {e}"))?;
    f.write_all(chunk)
        .await
        .map_err(|e| format!("write log: {e}"))?;
    Ok(())
}

pub async fn read_job_log(log_dir: &Path, run_id: &str, job_id: &str) -> Result<Vec<u8>, String> {
    let path = job_log_path(log_dir, run_id, job_id)?;
    fs::read(&path)
        .await
        .map_err(|e| format!("read log: {e}"))
}
