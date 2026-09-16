//! Phase 19 Wave 0 stubs — session RPC list/detail/logs (ACT-03 / D-ACT-12 / D-ACT-13).
//! Greened by later Actions UI/RPC plans.

#![allow(dead_code)]

/// Session RPC lists workflow runs for a visible repo (Read+).
#[tokio::test]
async fn actions_rpc_list_runs() {
    assert!(
        false,
        "expected actions.listRuns (or equivalent) for visible repos (ACT-03 / D-ACT-18)"
    );
}

/// Session RPC returns run detail including job list and statuses.
#[tokio::test]
async fn actions_rpc_run_detail() {
    assert!(
        false,
        "expected actions.getRun detail with jobs (D-ACT-12)"
    );
}

/// Session RPC returns job logs stored under OCTANEST_ACTIONS_LOG_DIR.
#[tokio::test]
async fn actions_rpc_job_logs() {
    assert!(
        false,
        "expected actions.getJobLogs from OCTANEST_ACTIONS_LOG_DIR (D-ACT-13)"
    );
}
