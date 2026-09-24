//! Persisted runner registration — survives restarts so the runner only
//! consumes a registration token once.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerState {
    pub runner_id: String,
    pub token: String,
    #[serde(default)]
    pub name: Option<String>,
}

pub fn load(path: &Path) -> Option<RunnerState> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn save(path: &Path, state: &RunnerState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = tmp_path(path);
    let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut p = path.as_os_str().to_os_string();
    p.push(".tmp");
    PathBuf::from(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("runner.json");
        let st = RunnerState {
            runner_id: "id-1".into(),
            token: "ort_secret".into(),
            name: Some("r1".into()),
        };
        save(&path, &st).unwrap();
        assert_eq!(load(&path).unwrap(), st);
    }

    #[test]
    fn load_missing_or_corrupt_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(&dir.path().join("nope.json")).is_none());
        let bad = dir.path().join("bad.json");
        std::fs::write(&bad, b"{not json").unwrap();
        assert!(load(&bad).is_none());
    }
}
