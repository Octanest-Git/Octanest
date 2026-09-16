//! Phase 19 Wave 0 stubs — Gitea-compatible runner protocol (ACT-06 / D-ACT-07 / D-ACT-09).
//! Greened by 19-04 / 19-05. Auth: registration/runner tokens only — session Cookie ignored (D-ACT-18 / T-19-01).

#![allow(dead_code)]

/// Register with registration token yields persistent runner token.
#[tokio::test]
async fn actions_runner_protocol_register() {
    assert!(
        false,
        "expected Register with registration token (ACT-06 / D-ACT-08)"
    );
}

/// Declare advertises labels using label[:schema[:args]] mapping.
#[tokio::test]
async fn actions_runner_protocol_declare_labels() {
    assert!(
        false,
        "expected Declare with label[:schema[:args]] (D-ACT-09)"
    );
}

/// FetchTask returns a queued job whose runs-on labels match the runner.
#[tokio::test]
async fn actions_runner_protocol_fetch_task_label_match() {
    assert!(
        false,
        "expected FetchTask label match for runs-on (D-ACT-07 / D-ACT-09)"
    );
}

/// Session Cookie must not authenticate runner protocol routes.
#[tokio::test]
async fn actions_runner_protocol_ignores_session_cookie() {
    assert!(
        false,
        "expected Cookie ignored on /api/actions runner protocol (D-ACT-18 / T-19-01)"
    );
}
