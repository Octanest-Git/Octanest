//! Phase 19 Wave 0 stubs — registered-only dispatch / no in-process exec (ACT-07 / D-ACT-10 / D-ACT-20).
//! Greened by 19-04+. Jobs never execute inside octanest-api (T-19-02).

#![allow(dead_code)]

/// Jobs are only assigned to registered runners with matching labels.
#[tokio::test]
async fn actions_dispatch_policy_registered_runners_only() {
    assert!(
        false,
        "expected jobs only on registered runners (ACT-07 / D-ACT-10)"
    );
}

/// Unmatched runs-on labels remain queued — never silently execute on the API host.
#[tokio::test]
async fn actions_dispatch_policy_unmatched_labels_stay_queued() {
    assert!(
        false,
        "expected unmatched labels stay queued (D-ACT-20 / T-19-02)"
    );
}

/// No managed Octanest Cloud minutes / in-process job execution in v1.
#[tokio::test]
async fn actions_dispatch_policy_no_in_process_execution() {
    assert!(
        false,
        "expected no in-process job execution on API host (ACT-07 / D-ACT-10)"
    );
}
