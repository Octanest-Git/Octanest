//! Actions control plane (Phase 19) — parse/discover only; no in-process job execution.

pub mod parse;
pub mod workflow;

pub use parse::{
    parse_workflow_yaml, JobSpec, ParseError, StepSpec, WorkflowDocument, WorkflowTriggers,
};
pub use workflow::{discover_workflows, DiscoverError, DiscoveredWorkflow, MAX_WORKFLOW_BYTES};
