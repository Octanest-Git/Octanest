//! In-process russh Git SSH listener (D-SSH-01 / D-SSH-03).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use octanest_db::Database;
use russh::server::{Auth, ChannelOpenHandle, Handler, Msg, Server as RusshServer, Session};
use russh::{Channel, ChannelId, MethodKind, MethodSet};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::auth;
use super::host_keys;
use super::pack::{self, PackCommand};

/// Shared state for all SSH connections.
#[derive(Clone)]
pub struct SshState {
    pub db: Database,
    pub repos_dir: PathBuf,
}

#[derive(Clone)]
struct SshServer {
    state: SshState,
}

struct SshHandler {
    state: SshState,
    user_id: Option<String>,
    key_id: Option<String>,
    session_channel: Option<Channel<Msg>>,
}

impl RusshServer for SshServer {
    type Handler = SshHandler;

    fn new_client(&mut self, _: Option<SocketAddr>) -> Self::Handler {
        SshHandler {
            state: self.state.clone(),
            user_id: None,
            key_id: None,
            session_channel: None,
        }
    }
}

impl Handler for SshHandler {
    type Error = russh::Error;

    async fn auth_publickey(
        &mut self,
        user: &str,
        public_key: &ssh_key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        if user != "git" {
            tracing::debug!(%user, "SSH reject: username must be git");
            return Ok(Auth::Reject {
                proceed_with_methods: None,
                partial_success: false,
            });
        }

        match auth::find_registered_key(&self.state.db, public_key).await {
            Ok(Some(row)) => {
                self.user_id = Some(row.user_id.clone());
                self.key_id = Some(row.id.clone());
                let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                let _ = self
                    .state
                    .db
                    .touch_ssh_key_last_used(&row.id, &now, None)
                    .await;
                Ok(Auth::Accept)
            }
            Ok(None) => Ok(Auth::Reject {
                proceed_with_methods: None,
                partial_success: false,
            }),
            Err(e) => {
                tracing::error!(error = %e, "SSH fingerprint lookup failed");
                Ok(Auth::Reject {
                    proceed_with_methods: None,
                    partial_success: false,
                })
            }
        }
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        reply: ChannelOpenHandle,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.session_channel = Some(channel);
        reply.accept().await;
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let Some(user_id) = self.user_id.clone() else {
            session.channel_failure(channel)?;
            return Ok(());
        };

        let parsed = pack::parse_pack_exec(data);
        match parsed {
            Some(PackCommand::UploadPack { owner, name }) => {
                let bare = match pack::resolve_bare(&self.state.repos_dir, &owner, &name) {
                    Ok(p) => p,
                    Err(_) => {
                        session.channel_failure(channel)?;
                        return Ok(());
                    }
                };
                if !bare.exists() {
                    session.channel_failure(channel)?;
                    return Ok(());
                }

                // Tracer ACL: allow upload-pack for any authenticated key (public repos).
                // Private/non-owner deny lands in 09-04.
                let _ = user_id;
                session.channel_success(channel)?;

                let Some(mut ch) = self.session_channel.take() else {
                    session.channel_failure(channel)?;
                    return Ok(());
                };

                let handle = session.handle();
                tokio::spawn(async move {
                    // Writers are `'static`; create them before the mut reader borrow.
                    let writer = ch.make_writer();
                    let stderr_writer = ch.make_writer_ext(Some(1));
                    let reader = ch.make_reader();
                    let code = pack::run_upload_pack(&bare, reader, writer, stderr_writer)
                        .await
                        .unwrap_or(1);
                    let _ = handle.exit_status_request(channel, code as u32).await;
                    let _ = handle.eof(channel).await;
                    let _ = handle.close(channel).await;
                });
                Ok(())
            }
            Some(PackCommand::ReceivePack { .. }) => {
                // Tracer: receive-pack deferred to 09-04.
                session.channel_failure(channel)?;
                Ok(())
            }
            None => {
                session.channel_failure(channel)?;
                Ok(())
            }
        }
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(channel)?;
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        _col_width: u32,
        _row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(channel)?;
        Ok(())
    }

    async fn subsystem_request(
        &mut self,
        channel: ChannelId,
        _name: &str,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(channel)?;
        Ok(())
    }
}

/// True when `OCTANEST_SSH_ENABLED` is `1`/`true`/`yes` (case-insensitive).
pub fn ssh_enabled_from_env() -> bool {
    std::env::var("OCTANEST_SSH_ENABLED")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

pub fn ssh_port_from_env() -> u16 {
    std::env::var("OCTANEST_SSH_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2222)
}

/// Bind and serve SSH until stopped. Returns bound address.
pub async fn spawn_listener(
    state: SshState,
    bind: SocketAddr,
) -> Result<(SocketAddr, oneshot::Sender<()>), String> {
    let host_dir = host_keys::host_key_dir();
    let host_key = host_keys::load_or_generate(&host_dir).await?;

    let config = russh::server::Config {
        inactivity_timeout: Some(Duration::from_secs(300)),
        auth_rejection_time: Duration::from_millis(10),
        auth_rejection_time_initial: Some(Duration::from_millis(0)),
        keys: vec![host_key],
        methods: MethodSet::from(&[MethodKind::PublicKey][..]),
        ..Default::default()
    };
    let config = Arc::new(config);
    let listener = TcpListener::bind(bind)
        .await
        .map_err(|e| format!("SSH bind {bind}: {e}"))?;
    let local = listener
        .local_addr()
        .map_err(|e| format!("SSH local_addr: {e}"))?;

    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let mut server = SshServer { state };
        let run = server.run_on_socket(config, &listener);
        let handle = run.handle();
        tokio::select! {
            _ = run => {}
            _ = stop_rx => {
                handle.shutdown("shutdown".into());
            }
        }
    });

    // Give the accept loop a tick to start.
    tokio::task::yield_now().await;

    tracing::info!(%local, "Git SSH listener ready");
    Ok((local, stop_tx))
}

/// Start SSH from env when enabled (production path).
pub async fn maybe_spawn_from_env(db: Database, repos_dir: PathBuf) -> Option<oneshot::Sender<()>> {
    if !ssh_enabled_from_env() {
        tracing::info!("OCTANEST_SSH_ENABLED not set; SSH listener skipped");
        return None;
    }
    let port = ssh_port_from_env();
    let bind: SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .expect("SSH bind parse");
    match spawn_listener(SshState { db, repos_dir }, bind).await {
        Ok((_addr, stop)) => Some(stop),
        Err(e) => {
            tracing::error!(error = %e, "failed to start SSH listener");
            None
        }
    }
}
