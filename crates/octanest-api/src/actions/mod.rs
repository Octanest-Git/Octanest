//! Actions control plane (Phase 19) — parse/discover/dispatch/protocol; no in-process job execution.

pub mod dispatch;
pub mod logs;
pub mod parse;
pub mod runner_proto;
pub mod tokens;
pub mod workflow;

pub use dispatch::{dispatch_push_for_sha, enqueue_run, notify_push_actions};
pub use logs::{append_job_log, read_job_log};
pub use parse::{
    parse_workflow_yaml, JobSpec, ParseError, StepSpec, WorkflowDocument, WorkflowTriggers,
};
pub use tokens::mint_registration_token;
pub use workflow::{discover_workflows, DiscoverError, DiscoveredWorkflow, MAX_WORKFLOW_BYTES};
