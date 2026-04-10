use crate::push_handler;
use anyhow::Result;
use async_trait::async_trait;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec};
use ssh_key::{Algorithm, HashAlg, PrivateKey, PublicKey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

        let config = russh::server::Config { keys: vec![key], ..Default::default() };

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
        }
    }
}

pub struct SshSession {
    contract: Arc<ContractAbiClient>,
    authenticated_fingerprint: Option<SshKeyFingerprint>,
    /// Stdin writers for active git-receive-pack subprocesses, keyed by channel ID.
    stdin_writers: Arc<Mutex<HashMap<ChannelId, tokio::process::ChildStdin>>>,
}

impl SshSession {
    fn compute_fingerprint(key: &PublicKey) -> SshKeyFingerprint {
        let fp = key.fingerprint(HashAlg::Sha256);
        let bytes = fp.sha256().expect("sha256 fingerprint");
        SshKeyFingerprint(bytes)
    }

    fn parse_repo_name(command: &str) -> Option<String> {
        let command = command.trim();
        let suffix = command.strip_prefix("git-receive-pack ")?;
        let repo = suffix.trim_matches('\'').trim_matches('"').trim_matches('/');
        let repo = repo.strip_suffix(".git").unwrap_or(repo);
        if repo.is_empty() {
            return None;
        }
        Some(repo.to_string())
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
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn data(
        &mut self,
        channel_id: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Pipe incoming SSH channel data to the git-receive-pack subprocess stdin
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
        // Client done sending — close stdin so git-receive-pack can finish
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

        // Only allow git-receive-pack (push)
        let repo_name = match Self::parse_repo_name(command) {
            Some(name) => name,
            None => {
                let msg = format!("unsupported command: {}\r\n", command);
                let _ = session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                let _ = session.close(channel_id);
                return Ok(());
            }
        };

        // Authorization: check SSH key fingerprint against on-chain repo
        let fingerprint = match &self.authenticated_fingerprint {
            Some(fp) => fp.clone(),
            None => {
                let _ = session.extended_data(
                    channel_id,
                    1,
                    CryptoVec::from(b"error: not authenticated\r\n" as &[u8]),
                );
                let _ = session.close(channel_id);
                return Ok(());
            }
        };

        let state = self.contract.state().await;
        let repo_info = match state.repo_store.get(&repo_name).await {
            Some(info) => info,
            None => {
                let msg = format!("error: repository '{}' not found\r\n", repo_name);
                let _ = session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                let _ = session.close(channel_id);
                return Ok(());
            }
        };

        //verify ssh key matches repos owner ssh key
        match &repo_info.ssh_key_fingerprint {
            Some(registered) if registered.0 == fingerprint.0 => {}
            Some(_) => {
                let _ = session.extended_data(
                    channel_id,
                    1,
                    CryptoVec::from(
                        b"error: your SSH key is not authorized for this repository\r\n" as &[u8],
                    ),
                );
                let _ = session.close(channel_id);
                return Ok(());
            }
            None => {
                let _ = session.extended_data(
                    channel_id,
                    1,
                    CryptoVec::from(
                        b"error: no SSH key registered for this repository. Set one via the gitter web interface.\r\n" as &[u8],
                    ),
                );
                let _ = session.close(channel_id);
                return Ok(());
            }
        }

        // Materialize temp bare repo from chain
        let _ = session.extended_data(
            channel_id,
            1,
            CryptoVec::from(b"remote: Preparing repository...\r\n" as &[u8]),
        );

        let tmp = match tempfile::tempdir() {
            Ok(t) => t,
            Err(e) => {
                let msg = format!("error: failed to prepare repo: {}\r\n", e);
                let _ = session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                let _ = session.close(channel_id);
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
                let msg = format!("error: failed to prepare repo: {}\r\n", e);
                let _ = session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                let _ = session.close(channel_id);
                return Ok(());
            }
            Err(e) => {
                let msg = format!("error: internal error: {}\r\n", e);
                let _ = session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                let _ = session.close(channel_id);
                return Ok(());
            }
        }

        // Spawn git-receive-pack subprocess
        let repo_path_str = repo_path.to_string_lossy().to_string();
        let mut child = tokio::process::Command::new("git")
            .args(["receive-pack", &repo_path_str])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Store stdin writer so data() handler can pipe to it
        let child_stdin = child.stdin.take().unwrap();
        {
            let mut writers = self.stdin_writers.lock().await;
            writers.insert(channel_id, child_stdin);
        }

        // Pipe stdout/stderr to SSH channel, then sync to chain
        let contract = self.contract.clone();
        let ssh_handle = session.handle();
        let stdin_writers = self.stdin_writers.clone();

        tokio::spawn(async move {
            let result = drive_receive_pack(
                child,
                &repo_path,
                &repo_name,
                channel_id,
                &ssh_handle,
                &contract,
                &stdin_writers,
                tmp, // keep temp dir alive until done
            )
            .await;
            if let Err(e) = &result {
                tracing::error!(repo = repo_name, error = %e, "receive-pack failed");
                let msg = format!("error: {}\r\n", e);
                let _ =
                    ssh_handle.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes())).await;
            }
            let exit_code = if result.is_ok() { 0 } else { 1 };
            let _ = ssh_handle.exit_status_request(channel_id, exit_code).await;
            let _ = ssh_handle.eof(channel_id).await;
            let _ = ssh_handle.close(channel_id).await;
        });

        Ok(())
    }
}

/// Drive the git-receive-pack subprocess: pipe stdout/stderr to SSH, wait for exit,
/// then sync new objects to chain.
async fn drive_receive_pack(
    mut child: tokio::process::Child,
    repo_path: &std::path::Path,
    repo_name: &str,
    channel_id: ChannelId,
    handle: &russh::server::Handle,
    contract: &ContractAbiClient,
    stdin_writers: &Mutex<HashMap<ChannelId, tokio::process::ChildStdin>>,
    _tmp: tempfile::TempDir, // kept alive for the duration
) -> Result<()> {
    // Pipe stdout to SSH channel
    let mut stdout = child.stdout.take().unwrap();
    let handle_out = handle.clone();
    let stdout_task = tokio::spawn(async move {
        let mut buf = [0u8; 8192];
        loop {
            match stdout.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let _ = handle_out.data(channel_id, CryptoVec::from(&buf[..n])).await;
                }
                Err(_) => break,
            }
        }
    });

    // Pipe stderr to SSH channel
    let mut stderr = child.stderr.take().unwrap();
    let handle_err = handle.clone();
    let stderr_task = tokio::spawn(async move {
        let mut buf = [0u8; 8192];
        loop {
            match stderr.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let _ =
                        handle_err.extended_data(channel_id, 1, CryptoVec::from(&buf[..n])).await;
                }
                Err(_) => break,
            }
        }
    });

    // Wait for subprocess to exit (stdin is closed by channel_eof or when client disconnects)
    let status = child.wait().await?;

    // Clean up stdin writer
    {
        let mut writers = stdin_writers.lock().await;
        writers.remove(&channel_id);
    }

    // Wait for stdout/stderr forwarding to finish
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    if !status.success() {
        anyhow::bail!("git receive-pack exited with {}", status);
    }

    // Sync to chain with progress messages
    tracing::info!(repo = repo_name, "receive-pack complete, syncing to chain");
    send_remote_msg(handle, channel_id, "Collecting objects...").await;

    let collected = push_handler::collect_new_objects(repo_path, repo_name, contract).await?;

    let (local_head, objects) = match collected {
        push_handler::CollectedObjects::AlreadyUpToDate => {
            send_remote_msg(handle, channel_id, "Already up to date on chain").await;
            return Ok(());
        }
        push_handler::CollectedObjects::New { local_head, objects } => (local_head, objects),
    };

    let count = objects.len();
    send_remote_msg(
        handle,
        channel_id,
        &format!("Uploading to Vastrum chain... ({} objects)", count),
    )
    .await;

    let uploaded = push_handler::upload_objects(&objects, contract).await?;

    send_remote_msg(
        handle,
        channel_id,
        &format!("Uploaded {} objects, updating HEAD...", uploaded),
    )
    .await;

    push_handler::update_and_verify_head(repo_name, local_head, contract).await?;

    send_remote_msg(handle, channel_id, &format!("Synced to Vastrum chain ({} objects)", uploaded))
        .await;

    tracing::info!(repo = repo_name, objects = uploaded, head = %local_head, "pushed to chain");
    Ok(())
}

async fn send_remote_msg(handle: &russh::server::Handle, channel_id: ChannelId, msg: &str) {
    let formatted = format!("remote: {}\r\n", msg);
    let _ = handle.extended_data(channel_id, 1, CryptoVec::from(formatted.as_bytes())).await;
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
