//! Phase 12 integration hooks for Actions (D-ACT-05).
//!
//! Call sites required once Phase 12 PR lifecycle lands (or is wired on this branch):
//!
//! | PR lifecycle | `PullRequestAction` | Call |
//! |--------------|---------------------|------|
//! | PR opened | `Opened` | `actions::dispatch_pull_request` |
//! | PR head updated | `Synchronize` | same |
//! | PR reopened | `Reopened` | same |
//!
//! Do **not** expose a public HTTP endpoint for these events (T-19-13).

pub use crate::actions::events::{
    dispatch_pull_request, dispatch_pull_request_for_sha, PullRequestAction, PullRequestEvent,
};

#[cfg(test)]
mod harness {
    use super::*;

    #[test]
    fn phase12_hook_symbols_are_invocable() {
        // Compile-time surface: Phase 12 imports these names from `actions::hooks`.
        assert_eq!(
            format!("{:?}", PullRequestAction::Opened),
            "Opened"
        );
        assert_eq!(
            format!("{:?}", PullRequestAction::Synchronize),
            "Synchronize"
        );
        assert_eq!(
            format!("{:?}", PullRequestAction::Reopened),
            "Reopened"
        );
        let _fn = dispatch_pull_request;
        let _fn2 = dispatch_pull_request_for_sha;
        let _ = std::mem::size_of_val(&_fn);
        let _ = std::mem::size_of_val(&_fn2);
        let _ = std::any::type_name::<PullRequestEvent>();
    }
}
