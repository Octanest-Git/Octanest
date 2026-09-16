//! Phase 19 Wave 0 stubs — workflow discovery/parse (ACT-01 / D-ACT-01 / D-ACT-02).
//! Greened by 19-03.

#![allow(dead_code)]

/// Discover workflows under `.github/workflows/*.{yml,yaml}` on the triggering ref.
#[tokio::test]
async fn actions_workflow_parse_discovers_github_workflows_path() {
    assert!(
        false,
        "expected discovery under .github/workflows/*.{{yml,yaml}} (ACT-01 / D-ACT-01)"
    );
}

/// Parse GitHub Actions–compatible YAML subset (name, on, jobs, runs-on, steps, env).
#[tokio::test]
async fn actions_workflow_parse_gha_compatible_subset() {
    assert!(
        false,
        "expected GHA-compatible YAML subset parse (D-ACT-02)"
    );
}

/// Unsupported constructs fail the job with a clear error (no parallel DSL).
#[tokio::test]
async fn actions_workflow_parse_unsupported_fails_clearly() {
    assert!(
        false,
        "expected unsupported constructs to fail the job clearly (D-ACT-02)"
    );
}
