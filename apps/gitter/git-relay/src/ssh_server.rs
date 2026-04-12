use crate::push_handler;
use anyhow::Result;
use async_trait::async_trait;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec};
use ssh_key::{Algorithm, HashAlg, PrivateKey, PublicKey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::SshKeyFingerprint;

pub struct SshServer {
    contract: Arc<ContractAbiClient>,
}

impl SshServer {
    pub fn new(contract: Arc<ContractAbiClient>) -> Self {
        Self { contract }
    }

    pub async fn run(mut self, port: u16, host_key_path: &std::path::Path) -> Result<()> {
        let key = load_or_generate_host_key(host_key_path)?;

        let config = russh::server::Config {
            keys: vec![key],
            keepalive_interval: Some(std::time::Duration::from_secs(15)),
            keepalive_max: 3,
            ..Default::default()
        };

        let config = Arc::new(config);
        tracing::info!(port, "SSH server listening");
        use russh::server::Server as _;
        self.run_on_address(config, ("0.0.0.0", port)).await?;
        Ok(())
    }
}

impl russh::server::Server for SshServer {
    type Handler = SshSession;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        SshSession {
            contract: self.contract.clone(),
            authenticated_fingerprint: None,
            stdin_writers: Arc::new(Mutex::new(HashMap::new())),
            channels: HashMap::new(),
        }
    }
}

pub struct SshSession {
    contract: Arc<ContractAbiClient>,
    authenticated_fingerprint: Option<SshKeyFingerprint>,
    /// Stdin writers for active git subprocesses, keyed by channel ID.
    stdin_writers: Arc<Mutex<HashMap<ChannelId, tokio::process::ChildStdin>>>,
    /// Channels stored from channel_open_session, consumed by exec_request.
    channels: HashMap<ChannelId, Channel<Msg>>,
}

#[derive(Clone, Copy, Debug)]
enum GitService {
    UploadPack,
    ReceivePack,
}

impl GitService {
    fn git_subcommand(self) -> &'static str {
        match self {
            GitService::UploadPack => "upload-pack",
            GitService::ReceivePack => "receive-pack",
        }
    }
}

impl SshSession {
    fn compute_fingerprint(key: &PublicKey) -> SshKeyFingerprint {
        let fp = key.fingerprint(HashAlg::Sha256);
        let bytes = fp.sha256().expect("sha256 fingerprint");
        SshKeyFingerprint(bytes)
    }

    fn parse_git_command(command: &str) -> Option<(GitService, String)> {
        let command = command.trim();
        let (service, suffix) = if let Some(s) = command.strip_prefix("git-upload-pack ") {
            (GitService::UploadPack, s)
        } else if let Some(s) = command.strip_prefix("git-receive-pack ") {
            (GitService::ReceivePack, s)
        } else {
            return None;
        };
        let repo = suffix.trim_matches('\'').trim_matches('"').trim_matches('/');
        let repo = repo.strip_suffix(".git").unwrap_or(repo);
        if repo.is_empty() {
            return None;
        }
        Some((service, repo.to_string()))
    }
}

#[async_trait]
impl Handler for SshSession {
    type Error = anyhow::Error;

    async fn auth_publickey(
        &mut self,
        _user: &str,
        public_key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        let fp = Self::compute_fingerprint(public_key);
        tracing::debug!(fingerprint = hex::encode(&fp.0), "SSH key authenticated");
        self.authenticated_fingerprint = Some(fp);
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        self.channels.insert(channel.id(), channel);
        Ok(true)
    }

    async fn data(
        &mut self,
        channel_id: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Pipe incoming SSH channel data to the active git subprocess stdin
        // (upload-pack or receive-pack, whichever the client invoked).
        let mut writers = self.stdin_writers.lock().await;
        if let Some(stdin) = writers.get_mut(&channel_id) {
            let _ = stdin.write_all(data).await;
        }
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        channel_id: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Client done sending — close stdin so the git subprocess can finish.
        let mut writers = self.stdin_writers.lock().await;
        writers.remove(&channel_id);
        Ok(())
    }

    async fn channel_close(
        &mut self,
        channel_id: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Client disconnected abruptly — clean up stdin so git subprocess exits.
        let mut writers = self.stdin_writers.lock().await;
        writers.remove(&channel_id);
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel_id: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = std::str::from_utf8(data)?;
        tracing::info!(command, "SSH exec request");

        let (service, repo_name) = match Self::parse_git_command(command) {
            Some(parsed) => parsed,
            None => {
                send_error_and_close(
                    session,
                    channel_id,
                    &format!("unsupported command: {}\r\n", command),
                );
                return Ok(());
            }
        };

        match service {
            GitService::UploadPack => {
                self.handle_upload_pack(channel_id, repo_name, session).await
            }
            GitService::ReceivePack => {
                self.handle_receive_pack(channel_id, repo_name, session).await
            }
        }
    }
}

impl SshSession {
    /// Anonymous clone/fetch. Verify the repo exists, then stream upload-pack.
    async fn handle_upload_pack(
        &mut self,
        channel_id: ChannelId,
        repo_name: String,
        session: &mut Session,
    ) -> Result<(), anyhow::Error> {
        let state = self.contract.state().await;
        if state.repo_store.get(&repo_name).await.is_none() {
            send_error_and_close(
                session,
                channel_id,
                &format!("error: repository '{}' not found\r\n", repo_name),
            );
            return Ok(());
        }

        self.start_git_session(GitService::UploadPack, channel_id, repo_name, session).await
    }

    /// Authenticated push. Require the session's SSH key to match the repo's
    /// registered fingerprint, then stream receive-pack + sync to chain.
    async fn handle_receive_pack(
        &mut self,
        channel_id: ChannelId,
        repo_name: String,
        session: &mut Session,
    ) -> Result<(), anyhow::Error> {
        let fingerprint = match &self.authenticated_fingerprint {
            Some(fp) => fp.clone(),
            None => {
                send_error_and_close(session, channel_id, "error: not authenticated\r\n");
                return Ok(());
            }
        };

        let state = self.contract.state().await;
        let repo_info = match state.repo_store.get(&repo_name).await {
            Some(info) => info,
            None => {
                send_error_and_close(
                    session,
                    channel_id,
                    &format!("error: repository '{}' not found\r\n", repo_name),
                );
                return Ok(());
            }
        };

        match &repo_info.ssh_key_fingerprint {
            Some(registered) if registered.0 == fingerprint.0 => {}
            Some(_) => {
                send_error_and_close(
                    session,
                    channel_id,
                    "error: your SSH key is not authorized for this repository\r\n",
                );
                return Ok(());
            }
            None => {
                send_error_and_close(
                    session,
                    channel_id,
                    "error: no SSH key registered for this repository. Set one via the gitter web interface.\r\n",
                );
                return Ok(());
            }
        }

        self.start_git_session(GitService::ReceivePack, channel_id, repo_name, session).await
    }

    /// Shared body for both services: materialize the repo, spawn the git
    /// subprocess, register its stdin with the SSH session, and fire off a
    /// task that drives it to completion.
    async fn start_git_session(
        &mut self,
        service: GitService,
        channel_id: ChannelId,
        repo_name: String,
        session: &mut Session,
    ) -> Result<(), anyhow::Error> {
        let _ = session.extended_data(
            channel_id,
            1,
            CryptoVec::from(b"remote: Preparing repository...\r\n" as &[u8]),
        );

        let tmp = match tempfile::tempdir() {
            Ok(t) => t,
            Err(e) => {
                send_error_and_close(
                    session,
                    channel_id,
                    &format!("error: failed to prepare repo: {}\r\n", e),
                );
                return Ok(());
            }
        };
        let repo_path = tmp.path().join(format!("{}.git", repo_name));

        let site_id = self.contract.site_id();
        let repo_name_clone = repo_name.clone();
        let repo_path_clone = repo_path.clone();
        let handle = tokio::runtime::Handle::current();

        let materialize_result = tokio::task::spawn_blocking(move || {
            let contract = ContractAbiClient::new(site_id);
            handle.block_on(async {
                crate::materialize::materialize_bare_repo(
                    &repo_path_clone,
                    &repo_name_clone,
                    &contract,
                )
                .await
            })
        })
        .await;

        match materialize_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                send_error_and_close(
                    session,
                    channel_id,
                    &format!("error: failed to prepare repo: {}\r\n", e),
                );
                return Ok(());
            }
            Err(e) => {
                send_error_and_close(
                    session,
                    channel_id,
                    &format!("error: internal error: {}\r\n", e),
                );
                return Ok(());
            }
        }

        let repo_path_str = repo_path.to_string_lossy().to_string();
        let mut child = match tokio::process::Command::new("git")
            .args([service.git_subcommand(), &repo_path_str])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                send_error_and_close(
                    session,
                    channel_id,
                    &format!("error: failed to spawn git: {}\r\n", e),
                );
                return Ok(());
            }
        };

        let child_stdin = child.stdin.take().unwrap();
        {
            let mut writers = self.stdin_writers.lock().await;
            writers.insert(channel_id, child_stdin);
        }

        let channel = match self.channels.remove(&channel_id) {
            Some(ch) => ch,
            None => {
                send_error_and_close(session, channel_id, "error: no channel for exec\r\n");
                return Ok(());
            }
        };
        let contract = self.contract.clone();
        let ssh_handle = session.handle();
        let stdin_writers = self.stdin_writers.clone();

        tokio::spawn(async move {
            let result = drive_git_session(
                child,
                service,
                &repo_path,
                &repo_name,
                &channel,
                &contract,
                &stdin_writers,
                tmp,
            )
            .await;
            if let Err(e) = &result {
                tracing::error!(
                    repo = repo_name,
                    service = service.git_subcommand(),
                    error = %e,
                    "git session failed"
                );
                let _ = channel
                    .extended_data(1, std::io::Cursor::new(format!("error: {}\r\n", e).into_bytes()))
                    .await;
            }
            let exit_code = if result.is_ok() { 0 } else { 1 };
            let _ = ssh_handle.exit_status_request(channel_id, exit_code).await;
            let _ = channel.eof().await;
            let _ = channel.close().await;
        });

        Ok(())
    }
}

fn send_error_and_close(session: &mut Session, channel_id: ChannelId, msg: &str) {
    let _ = session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
    let _ = session.close(channel_id);
}

/// Drive a git subprocess (upload-pack or receive-pack) over an SSH channel:
/// pipe stdout/stderr, wait for exit, and for receive-pack, sync new objects to
/// chain afterwards.
async fn drive_git_session(
    mut child: tokio::process::Child,
    service: GitService,
    repo_path: &std::path::Path,
    repo_name: &str,
    channel: &Channel<Msg>,
    contract: &ContractAbiClient,
    stdin_writers: &Mutex<HashMap<ChannelId, tokio::process::ChildStdin>>,
    _tmp: tempfile::TempDir, // kept alive for the duration
) -> Result<()> {
    drive_git_subprocess(&mut child, service, channel, stdin_writers).await?;

    match service {
        GitService::UploadPack => Ok(()),
        GitService::ReceivePack => {
            sync_push_to_chain(repo_path, repo_name, contract, channel).await
        }
    }
}

/// Pipe a git subprocess's stdout/stderr to the SSH channel and wait for exit.
/// Uses the Channel API which provides window-aware AsyncWrite — data is only
/// sent when the client's window has space, avoiding russh's pending_data queue.
async fn drive_git_subprocess(
    child: &mut tokio::process::Child,
    service: GitService,
    channel: &Channel<Msg>,
    stdin_writers: &Mutex<HashMap<ChannelId, tokio::process::ChildStdin>>,
) -> Result<()> {
    let channel_id = channel.id();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let mut stdout_writer = channel.make_writer();
    let mut stderr_writer = channel.make_writer_ext(Some(1));

    let pipe_task = tokio::spawn(async move {
        let (r1, r2) = tokio::join!(
            tokio::io::copy(&mut stdout, &mut stdout_writer),
            tokio::io::copy(&mut stderr, &mut stderr_writer),
        );
        if let Err(e) = r1 {
            tracing::debug!("stdout pipe: {}", e);
        }
        if let Err(e) = r2 {
            tracing::debug!("stderr pipe: {}", e);
        }
    });

    // Wait for subprocess to exit (stdin is closed by channel_eof or when client disconnects)
    let status = child.wait().await?;

    // Clean up stdin writer
    {
        let mut writers = stdin_writers.lock().await;
        writers.remove(&channel_id);
    }

    // Wait for pipe forwarding to finish
    let _ = pipe_task.await;

    if !status.success() {
        anyhow::bail!("git {} exited with {}", service.git_subcommand(), status);
    }
    Ok(())
}

/// Receive-pack tail: walk the materialized repo, upload new objects, and
/// apply branch updates on chain.
async fn sync_push_to_chain(
    repo_path: &std::path::Path,
    repo_name: &str,
    contract: &ContractAbiClient,
    channel: &Channel<Msg>,
) -> Result<()> {
    tracing::info!(repo = repo_name, "receive-pack complete, syncing to chain");
    send_remote_msg(channel, "Collecting objects...").await;

    let plan = push_handler::collect_push_plan(repo_path, repo_name, contract).await?;

    if plan.updates.is_empty() && plan.deletes.is_empty() {
        send_remote_msg(channel, "Already up to date on chain").await;
        return Ok(());
    }

    let count = plan.objects.len();
    send_remote_msg(
        channel,
        &format!("Uploading to Vastrum chain... ({} objects)", count),
    )
    .await;

    let counter = std::sync::atomic::AtomicUsize::new(0);
    let upload_fut = push_handler::upload_objects(&plan.objects, contract, Some(&counter));
    tokio::pin!(upload_fut);
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    let uploaded = loop {
        tokio::select! {
            result = &mut upload_fut => {
                break result?;
            }
            _ = interval.tick() => {
                let done = counter.load(std::sync::atomic::Ordering::Relaxed);
                send_remote_msg(channel, &format!("Uploading objects: {}/{}...", done, count)).await;
            }
        }
    };

    send_remote_msg(
        channel,
        &format!("Uploaded {} objects, updating branches...", uploaded),
    )
    .await;

    push_handler::apply_branch_updates(repo_name, &plan, contract).await?;

    let summary = if plan.deletes.is_empty() {
        format!("Synced {} branches ({} objects)", plan.updates.len(), uploaded)
    } else {
        format!(
            "Synced {} branches, deleted {} ({} objects)",
            plan.updates.len(),
            plan.deletes.len(),
            uploaded
        )
    };
    send_remote_msg(channel, &summary).await;

    tracing::info!(
        repo = repo_name,
        objects = uploaded,
        updates = plan.updates.len(),
        deletes = plan.deletes.len(),
        "pushed to chain"
    );
    Ok(())
}

async fn send_remote_msg(channel: &Channel<Msg>, msg: &str) {
    let formatted = format!("remote: {}\r\n", msg);
    let _ = channel
        .extended_data(1, std::io::Cursor::new(formatted.into_bytes()))
        .await;
}

fn load_or_generate_host_key(path: &std::path::Path) -> Result<PrivateKey> {
    if path.exists() {
        let key = PrivateKey::read_openssh_file(path)?;
        tracing::info!("loaded SSH host key from {:?}", path);
        Ok(key)
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let key = PrivateKey::random(&mut rand::thread_rng(), Algorithm::Ed25519)?;
        key.write_openssh_file(path, ssh_key::LineEnding::LF)?;
        tracing::info!("generated new SSH host key at {:?}", path);
        Ok(key)
    }
}
