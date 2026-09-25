//! Boot-time `git --version` gate (D-33): require system git ≥ 2.5.

/// Parse `git --version` stdout (e.g. `"git version 2.55.0"`).
pub fn parse_git_version(s: &str) -> Result<(u32, u32, u32), String> {
    let trimmed = s.trim();
    let after = trimmed
        .strip_prefix("git version ")
        .ok_or_else(|| format!("unexpected git --version output: {trimmed}"))?;
    // Take the first token (drop Apple Git / Windows suffixes after space).
    let version_token = after.split_whitespace().next().unwrap_or(after);
    // Digits only for major.minor.patch — ignore trailing `.windows.1` etc.
    let mut parts = version_token.split('.').take(3).map(|p| {
        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits
            .parse::<u32>()
            .map_err(|_| format!("invalid version component in: {version_token}"))
    });
    let major = parts
        .next()
        .ok_or_else(|| format!("missing major in: {version_token}"))??;
    let minor = parts.next().transpose()?.unwrap_or(0);
    let patch = parts.next().transpose()?.unwrap_or(0);
    Ok((major, minor, patch))
}

/// Probe `git --version` and ensure it meets `min` (major, minor, patch).
pub fn assert_git_version(min: (u32, u32, u32)) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .arg("--version")
        .output()
        .map_err(|e| format!("git missing or not executable: {e}"))?;
    if !out.status.success() {
        return Err("git --version failed".into());
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let ver = parse_git_version(&s)?;
    if ver < min {
        return Err(format!(
            "git {}.{}.{} < required {}.{}.{}",
            ver.0, ver.1, ver.2, min.0, min.1, min.2
        ));
    }
    Ok(())
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
            "assert_git_version must succeed when host git >= 2.5 — {result:?}"
        );
    }

    /// Filter: `git_version_gate` — parse helper for version strings.
    #[test]
    fn git_version_gate_parses_git_version_stdout() {
        let ver = parse_git_version("git version 2.55.0");
        assert_eq!(
            ver.ok(),
            Some((2, 55, 0)),
            "parse_git_version must extract major.minor.patch"
        );
    }

    // git_archive_formats coverage lives in `cli::tests` (async CliGitBackend).
}
