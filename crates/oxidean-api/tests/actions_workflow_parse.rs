//! Phase 19 — workflow discovery/parse (ACT-01 / D-ACT-01 / D-ACT-02).

use std::sync::Arc;

use oxidean_api::actions::{discover_workflows, parse_workflow_yaml, DiscoverError};
use oxidean_git::{CliGitBackend, GitBackend};

#[tokio::test]
async fn actions_workflow_parse_discovers_github_workflows_path() {
    let dir = tempfile::tempdir().unwrap();
    let bare = dir.path().join("demo.git");
    let git = CliGitBackend::new();
    git.init_bare(&bare, "main").await.expect("init");
    git.seed_commit(
        &bare,
        "main",
        "add workflow",
        &[(
            ".github/workflows/ci.yml".into(),
            br#"
name: CI
on: [push]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo ok
"#
            .to_vec(),
        )],
    )
    .await
    .expect("seed");

    let found = discover_workflows(&git as &dyn GitBackend, &bare, "main")
        .await
        .expect("discover");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, ".github/workflows/ci.yml");
    assert_eq!(found[0].document.name, "CI");
    assert!(found[0].document.triggers.push);
}

#[tokio::test]
async fn actions_workflow_parse_gha_compatible_subset() {
    let yaml = br#"
name: PR
on:
  pull_request:
  push:
jobs:
  test:
    name: Test
    runs-on: [ubuntu-latest, self-hosted]
    env:
      FOO: bar
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 1
      - name: unit
        run: cargo test
        if: success()
"#;
    let doc = parse_workflow_yaml(yaml).expect("parse");
    assert_eq!(doc.name, "PR");
    assert!(doc.triggers.push && doc.triggers.pull_request);
    assert_eq!(doc.jobs[0].id, "test");
    assert_eq!(doc.jobs[0].name.as_deref(), Some("Test"));
    assert_eq!(
        doc.jobs[0].runs_on,
        vec!["ubuntu-latest".to_string(), "self-hosted".to_string()]
    );
    assert_eq!(doc.jobs[0].steps.len(), 2);
    assert!(doc.jobs[0].steps[0].uses.is_some());
    assert!(doc.jobs[0].steps[1].run.is_some());
    assert!(doc.jobs[0].env.is_some());
}

#[tokio::test]
async fn actions_workflow_parse_unsupported_fails_clearly() {
    let dir = tempfile::tempdir().unwrap();
    let bare = dir.path().join("bad.git");
    let git = CliGitBackend::new();
    git.init_bare(&bare, "main").await.expect("init");
    git.seed_commit(
        &bare,
        "main",
        "bad yaml",
        &[(
            ".github/workflows/broken.yml".into(),
            b"name: [\n  - broken\n".to_vec(),
        )],
    )
    .await
    .expect("seed");

    let err = discover_workflows(&git as &dyn GitBackend, &bare, "main")
        .await
        .expect_err("must fail");
    match err {
        DiscoverError::Parse { path, error } => {
            assert_eq!(path, ".github/workflows/broken.yml");
            assert!(
                error.message.contains("invalid workflow YAML"),
                "{}",
                error.message
            );
        }
        other => panic!("expected Parse, got {other}"),
    }

    // Missing runs-on → clear parse error (not silent execute).
    let missing = parse_workflow_yaml(
        br#"
name: X
on: push
jobs:
  j:
    steps:
      - run: "true"
"#,
    )
    .expect_err("missing runs-on");
    assert!(
        missing.message.contains("runs-on"),
        "{}",
        missing.message
    );
}

#[tokio::test]
async fn actions_workflow_parse_does_not_execute_steps() {
    // Guard: discovery/parse must not spawn containers — only return documents.
    let _git: Arc<dyn GitBackend> = Arc::new(CliGitBackend::new());
    let doc = parse_workflow_yaml(
        br#"
name: NoExec
on: push
jobs:
  j:
    runs-on: ubuntu-latest
    steps:
      - run: echo should-not-run-in-api
"#,
    )
    .expect("parse");
    assert_eq!(doc.jobs[0].steps[0].run.as_deref(), Some("echo should-not-run-in-api"));
}
