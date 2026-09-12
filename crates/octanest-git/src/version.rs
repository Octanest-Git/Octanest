//! Boot-time `git --version` gate (D-33): require system git ≥ 2.5.
//!
//! Wave 0: placeholders that fail until the implementation plan greens them.

/// Parse `git --version` stdout (e.g. `"git version 2.55.0"`).
pub fn parse_git_version(_s: &str) -> Result<(u32, u32, u32), String> {
    Err("Wave 0: parse_git_version not implemented".into())
}

/// Probe `git --version` and ensure it meets `min` (major, minor, patch).
pub fn assert_git_version(min: (u32, u32, u32)) -> Result<(), String> {
    let _ = min;
    Err("Wave 0: assert_git_version not implemented".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Filter: `git_version_gate` — boot rejects missing/old git (D-33).
    #[test]
    fn git_version_gate_rejects_below_floor() {
        let result = assert_git_version((2, 5, 0));
        assert!(
            result.is_ok(),
            "Wave 0: assert_git_version must succeed when host git >= 2.5 — {result:?}"
        );
    }

    /// Filter: `git_version_gate` — parse helper for version strings.
    #[test]
    fn git_version_gate_parses_git_version_stdout() {
        let ver = parse_git_version("git version 2.55.0");
        assert_eq!(
            ver.ok(),
            Some((2, 55, 0)),
            "Wave 0: parse_git_version must extract major.minor.patch"
        );
    }

    /// Filter: `git_archive_formats` — zip + tar.gz via CliGitBackend (GIT-07).
    #[test]
    fn git_archive_formats_zip_and_tar_gz() {
        assert!(
            false,
            "Wave 0: CliGitBackend::archive must support zip and tar.gz formats"
        );
    }
}
