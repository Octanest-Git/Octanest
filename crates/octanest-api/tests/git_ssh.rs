//! GIT-03 tracer: Git-over-SSH auth + public upload-pack (09-03).
//! Private/push/rate-limit cases stay RED until 09-04.

mod support;

use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::Arc;
use std::time::Duration;

use octanest_api::ssh::{spawn_listener, SshState};
use octanest_core::Role;
use octanest_db::Database;
use russh::client;
use russh::keys::{load_secret_key, PrivateKey, PrivateKeyWithHashAlg, PublicKeyOrCertificate};
use russh::keys::ssh_key::{Algorithm, HashAlg, LineEnding};
use ssh_key::PublicKey;
use tempfile::TempDir;
use uuid::Uuid;

struct AcceptingClient;

impl client::Handler for AcceptingClient {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

async fn connect_auth(
    addr: std::net::SocketAddr,
    user: &str,
    key: Arc<PrivateKey>,
) -> Result<client::Handle<AcceptingClient>, String> {
    let config = Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    });
    let mut session = client::connect(config, addr, AcceptingClient)
        .await
        .map_err(|e| format!("connect: {e}"))?;
    let ok = session
        .authenticate_publickey(
            user,
            PrivateKeyWithHashAlg::new(key, None),
        )
        .await
        .map_err(|e| format!("auth: {e}"))?;
    if !ok.success() {
        return Err("auth rejected".into());
    }
    Ok(session)
}

fn write_keypair(dir: &std::path::Path) -> (PathBuf, String, String) {
    let key = PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519).expect("gen key");
    let priv_path = dir.join("id_ed25519");
    let pem = key.to_openssh(LineEnding::LF).expect("encode");
    std::fs::write(&priv_path, pem.as_bytes()).expect("write priv");
    let pub_line = key.public_key().to_openssh().expect("pub");
    let fp = PublicKey::from_openssh(&pub_line)
        .expect("parse pub")
        .fingerprint(HashAlg::Sha256)
        .to_string();
    (priv_path, pub_line, fp)
}

fn init_bare_repo(bare: &std::path::Path) {
    std::fs::create_dir_all(bare.parent().unwrap()).expect("parent");
    let st = StdCommand::new("git")
        .args(["init", "--bare"])
        .arg(bare)
        .status()
        .expect("git init");
    assert!(st.success());
    // Seed one commit via a temp worktree push.
    let work = bare.parent().unwrap().join("seed-work");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    assert!(StdCommand::new("git")
        .args(["-C"])
        .arg(&work)
        .args(["init", "-b", "main"])
        .status()
        .unwrap()
        .success());
    assert!(StdCommand::new("git")
        .args(["-C"])
        .arg(&work)
        .args(["config", "user.email", "t@ex.com"])
        .status()
        .unwrap()
        .success());
    assert!(StdCommand::new("git")
        .args(["-C"])
        .arg(&work)
        .args(["config", "user.name", "t"])
        .status()
        .unwrap()
        .success());
    std::fs::write(work.join("README"), "hi\n").unwrap();
    assert!(StdCommand::new("git")
        .args(["-C"])
        .arg(&work)
        .args(["add", "README"])
        .status()
        .unwrap()
        .success());
    assert!(StdCommand::new("git")
        .args(["-C"])
        .arg(&work)
        .args(["commit", "-m", "init"])
        .status()
        .unwrap()
        .success());
    assert!(StdCommand::new("git")
        .args(["-C"])
        .arg(&work)
        .args(["remote", "add", "origin"])
        .arg(bare)
        .status()
        .unwrap()
        .success());
    assert!(StdCommand::new("git")
        .args(["-C"])
        .arg(&work)
        .args(["push", "-u", "origin", "main"])
        .status()
        .unwrap()
        .success());
}

/// SSH username other than `git` is rejected (D-SSH-03).
#[tokio::test]
async fn git_ssh_username_other_than_git_rejected() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("OCTANEST_SSH_HOST_KEY_DIR", tmp.path().join("host"));
    let url = format!("sqlite:{}", tmp.path().join("ssh.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let repos = tmp.path().join("repos");
    std::fs::create_dir_all(&repos).unwrap();

    let (priv_path, pub_line, fp) = write_keypair(tmp.path());
    let user_id = Uuid::new_v4().to_string();
    db.create_user(
        &user_id,
        "ssh@ex.com",
        "sshuser",
        Some("hash"),
        "SSH",
        "",
        None,
        Role::User,
    )
    .await
    .unwrap();
    db.create_ssh_key(
        "k1",
        &user_id,
        "laptop",
        &pub_line,
        &fp,
        "ssh-ed25519",
    )
    .await
    .unwrap();

    let (addr, _stop) = spawn_listener(
        SshState {
            db,
            repos_dir: repos,
        },
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("listen");

    let key = Arc::new(load_secret_key(&priv_path, None).unwrap());
    let err = connect_auth(addr, "notgit", key).await;
    assert!(err.is_err(), "non-git username must be rejected: {}", err.err().unwrap());
}

/// Registered public key + username `git` is accepted (D-SSH-03).
#[tokio::test]
async fn git_ssh_registered_key_user_git_accepted() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("OCTANEST_SSH_HOST_KEY_DIR", tmp.path().join("host"));
    let url = format!("sqlite:{}", tmp.path().join("ssh.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let repos = tmp.path().join("repos");
    std::fs::create_dir_all(&repos).unwrap();

    let (priv_path, pub_line, fp) = write_keypair(tmp.path());
    let user_id = Uuid::new_v4().to_string();
    db.create_user(
        &user_id,
        "ssh2@ex.com",
        "sshuser2",
        Some("hash"),
        "SSH",
        "",
        None,
        Role::User,
    )
    .await
    .unwrap();
    db.create_ssh_key(
        "k2",
        &user_id,
        "laptop",
        &pub_line,
        &fp,
        "ssh-ed25519",
    )
    .await
    .unwrap();

    let (addr, _stop) = spawn_listener(
        SshState {
            db,
            repos_dir: repos,
        },
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("listen");

    let key = Arc::new(load_secret_key(&priv_path, None).unwrap());
    connect_auth(addr, "git", key)
        .await
        .expect("registered key + git must authenticate");
}

/// Public repo: authenticated `git-upload-pack` happy path (D-SSH-04; A1 — key required).
#[tokio::test]
async fn git_ssh_public_upload_pack_happy_path() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("OCTANEST_SSH_HOST_KEY_DIR", tmp.path().join("host"));
    let url = format!("sqlite:{}", tmp.path().join("ssh.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let repos = tmp.path().join("repos");
    let bare = repos.join("sshuser3").join("demo.git");
    init_bare_repo(&bare);

    let (priv_path, pub_line, fp) = write_keypair(tmp.path());
    let user_id = Uuid::new_v4().to_string();
    db.create_user(
        &user_id,
        "ssh3@ex.com",
        "sshuser3",
        Some("hash"),
        "SSH",
        "",
        None,
        Role::User,
    )
    .await
    .unwrap();
    db.create_ssh_key(
        "k3",
        &user_id,
        "laptop",
        &pub_line,
        &fp,
        "ssh-ed25519",
    )
    .await
    .unwrap();

    let (addr, _stop) = spawn_listener(
        SshState {
            db,
            repos_dir: repos,
        },
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("listen");

    let key = Arc::new(load_secret_key(&priv_path, None).unwrap());
    let mut session = connect_auth(addr, "git", key).await.expect("auth");
    let mut channel = session
        .channel_open_session()
        .await
        .expect("open session");
    channel
        .exec(true, "git-upload-pack 'sshuser3/demo.git'")
        .await
        .expect("exec");

    // Read some pack advertisement / data — success if channel yields bytes or clean EOF.
    let mut saw_data = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), channel.wait()).await {
            Ok(Some(russh::ChannelMsg::Data { ref data })) if !data.is_empty() => {
                saw_data = true;
                break;
            }
            Ok(Some(russh::ChannelMsg::Eof)) => break,
            Ok(Some(russh::ChannelMsg::ExitStatus { exit_status })) => {
                assert_eq!(exit_status, 0, "upload-pack exit");
                saw_data = true;
                break;
            }
            Ok(None) => break,
            _ => {}
        }
    }
    assert!(
        saw_data,
        "expected git-upload-pack to produce data or exit 0"
    );
}

/// Private non-owner denied — deferred to 09-04.
#[tokio::test]
#[ignore = "09-04 ACL expansion"]
async fn git_ssh_private_non_owner_git_stderr_deny() {
    assert!(false);
}

/// Unverified push deny — deferred to 09-04.
#[tokio::test]
#[ignore = "09-04 ACL expansion"]
async fn git_ssh_push_unverified_email_denied() {
    assert!(false);
}

/// Non-pack exec rejected.
#[tokio::test]
async fn git_ssh_non_pack_exec_shell_rejected() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("OCTANEST_SSH_HOST_KEY_DIR", tmp.path().join("host"));
    let url = format!("sqlite:{}", tmp.path().join("ssh.db").display());
    let db = Database::connect(&url).await.expect("connect");
    db.migrate().await.expect("migrate");
    let repos = tmp.path().join("repos");
    std::fs::create_dir_all(&repos).unwrap();

    let (priv_path, pub_line, fp) = write_keypair(tmp.path());
    let user_id = Uuid::new_v4().to_string();
    db.create_user(
        &user_id,
        "ssh4@ex.com",
        "sshuser4",
        Some("hash"),
        "SSH",
        "",
        None,
        Role::User,
    )
    .await
    .unwrap();
    db.create_ssh_key(
        "k4",
        &user_id,
        "laptop",
        &pub_line,
        &fp,
        "ssh-ed25519",
    )
    .await
    .unwrap();

    let (addr, _stop) = spawn_listener(
        SshState {
            db,
            repos_dir: repos,
        },
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("listen");

    let key = Arc::new(load_secret_key(&priv_path, None).unwrap());
    let mut session = connect_auth(addr, "git", key).await.expect("auth");
    let mut channel = session.channel_open_session().await.expect("open");
    channel.exec(true, "bash").await.expect("exec sent");
    let mut failed = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(400), channel.wait()).await {
            Ok(Some(russh::ChannelMsg::Failure)) => {
                failed = true;
                break;
            }
            Ok(None) | Ok(Some(russh::ChannelMsg::Eof)) => break,
            _ => {}
        }
    }
    assert!(failed, "shell exec must receive channel failure");
}

/// Failed pubkey rate-limit — deferred to 09-04.
#[tokio::test]
#[ignore = "09-04 rate limit"]
async fn git_ssh_failed_pubkey_rate_limited() {
    assert!(false);
}
