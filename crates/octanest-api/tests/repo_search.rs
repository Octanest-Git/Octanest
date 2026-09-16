//! Phase 16 / GIT-18 Wave 0 stubs for `repo.search`.
//!
//! Filters (VALIDATION.md / D-SRCH-14):
//! - `repo_search_code` — code hits via git grep (D-SRCH-04, D-SRCH-06)
//! - `repo_search_commits` — commit message / author search (D-SRCH-07)
//! - `repo_search_issues` — issue title/body search (D-SRCH-09)
//! - `repo_search_pulls` — PR title/body search, separate from issues (D-SRCH-10, D-SRCH-11)
//! - `repo_search_acl` — private unauthorized → soft `repo.not_found` (D-SRCH-04)
//! - `repo_search_limits` — timeout / truncated soft caps (D-SRCH-08)
//!
//! Intentionally ignored until plans 16-01…16-03 turn them green. Do not implement
//! production `repo.search` handlers in Wave 0.

/// D-SRCH-06 / D-SRCH-14: Read user finds seeded text on default branch via type=code.
#[tokio::test]
#[ignore = "Wave 0 stub — greened in 16-01 (repo.search type=code)"]
async fn repo_search_code() {
    assert!(
        false,
        "Wave 0: repo.search type=code returns path+line hits for seeded string (D-SRCH-06)"
    );
}

/// D-SRCH-07 / D-SRCH-12: type=commits matches message grep and author:login.
#[tokio::test]
#[ignore = "Wave 0 stub — greened in 16-02 (repo.search type=commits)"]
async fn repo_search_commits() {
    assert!(
        false,
        "Wave 0: repo.search type=commits finds commits by message and author: (D-SRCH-07)"
    );
}

/// D-SRCH-09 / D-SRCH-12: type=issues matches title/body with is:open/is:closed.
#[tokio::test]
#[ignore = "Wave 0 stub — greened in 16-02 (repo.search type=issues)"]
async fn repo_search_issues() {
    assert!(
        false,
        "Wave 0: repo.search type=issues finds issues by title/body (D-SRCH-09)"
    );
}

/// D-SRCH-10 / D-SRCH-11: type=pulls finds PRs separately from issues.
#[tokio::test]
#[ignore = "Wave 0 stub — greened in 16-02 (repo.search type=pulls)"]
async fn repo_search_pulls() {
    assert!(
        false,
        "Wave 0: repo.search type=pulls finds PRs without mixing issues (D-SRCH-10)"
    );
}

/// D-SRCH-04: private unauthorized actor gets soft repo.not_found.
#[tokio::test]
#[ignore = "Wave 0 stub — greened in 16-01 (repo.search ACL soft not_found)"]
async fn repo_search_acl() {
    assert!(
        false,
        "Wave 0: private unauthorized repo.search → soft repo.not_found (D-SRCH-04)"
    );
}

/// D-SRCH-08: soft max-matches / timeout yield truncated or search.timeout.
#[tokio::test]
#[ignore = "Wave 0 stub — greened in 16-01/16-03 (caps + ENV timeout)"]
async fn repo_search_limits() {
    assert!(
        false,
        "Wave 0: repo.search respects soft caps / truncated flag (D-SRCH-08)"
    );
}
