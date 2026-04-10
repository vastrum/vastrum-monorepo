use crate::push_handler;
use crate::repo_cache::RepoCache;
use anyhow::Result;
use async_trait::async_trait;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec};
use ssh_key::{Algorithm, HashAlg, PrivateKey, PublicKey};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use vastrum_git_lib::SshKeyFingerprint;

pub struct SshServer {
    cache: Arc<RepoCache>,
}

impl SshServer {
    pub fn new(cache: Arc<RepoCache>) -> Self {
        Self { cache }
    }

    pub async fn run(mut self, port: u16, host_key_path: &std::path::Path) -> Result<()> {
        let key = load_or_generate_host_key(host_key_path)?;

        let config = russh::server::Config {
            keys: vec![key],
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
            cache: self.cache.clone(),
            authenticated_fingerprint: None,
        }
    }
}

pub struct SshSession {
    cache: Arc<RepoCache>,
    authenticated_fingerprint: Option<SshKeyFingerprint>,
}

impl SshSession {
    /// Compute the SHA256 fingerprint of an SSH public key.
    fn compute_fingerprint(key: &PublicKey) -> SshKeyFingerprint {
        let fp = key.fingerprint(HashAlg::Sha256);
        let bytes = fp.sha256().expect("sha256 fingerprint");
        SshKeyFingerprint(bytes)
    }

    /// Parse repo name from git-receive-pack command.
    fn parse_repo_name(command: &str) -> Option<String> {
        let command = command.trim();
        let suffix = command.strip_prefix("git-receive-pack ")?;
        let repo = suffix
            .trim_matches('\'')
            .trim_matches('"')
            .trim_matches('/');
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
                session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                session.close(channel_id);
                return Ok(());
            }
        };

        // Authorization: check SSH key fingerprint against on-chain repo
        let fingerprint = match &self.authenticated_fingerprint {
            Some(fp) => fp.clone(),
            None => {
                session.extended_data(
                    channel_id,
                    1,
                    CryptoVec::from(b"error: not authenticated\r\n" as &[u8]),
                );
                session.close(channel_id);
                return Ok(());
            }
        };

        let state = self.cache.contract().state().await;
        let repo_info = match state.repo_store.get(&repo_name).await {
            Some(info) => info,
            None => {
                let msg = format!("error: repository '{}' not found\r\n", repo_name);
                session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                session.close(channel_id);
                return Ok(());
            }
        };

        match &repo_info.ssh_key_fingerprint {
            Some(registered) if registered.0 == fingerprint.0 => {
                // Authorized
            }
            Some(_) => {
                session.extended_data(
                    channel_id,
                    1,
                    CryptoVec::from(
                        b"error: your SSH key is not authorized for this repository\r\n" as &[u8],
                    ),
                );
                session.close(channel_id);
                return Ok(());
            }
            None => {
                session.extended_data(
                    channel_id,
                    1,
                    CryptoVec::from(
                        b"error: no SSH key registered for this repository. Set one via the gitter web interface.\r\n" as &[u8],
                    ),
                );
                session.close(channel_id);
                return Ok(());
            }
        }

        // Ensure bare repo exists
        let repo_path = match self.cache.ensure_exists(&repo_name).await {
            Ok(path) => path,
            Err(e) => {
                let msg = format!("error: failed to prepare repo: {}\r\n", e);
                session.extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()));
                session.close(channel_id);
                return Ok(());
            }
        };

        // Spawn git-receive-pack as a subprocess and pipe I/O
        let cache = self.cache.clone();
        let handle = session.handle();

        tokio::spawn(async move {
            let result =
                run_receive_pack(&repo_path, &repo_name, channel_id, &handle, &cache).await;
            if let Err(e) = &result {
                tracing::error!(repo = repo_name, error = %e, "receive-pack failed");
                let msg = format!("error: {}\r\n", e);
                let _ = handle
                    .extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()))
                    .await;
            }
            let exit_code = if result.is_ok() { 0 } else { 1 };
            let _ = handle.exit_status_request(channel_id, exit_code).await;
            let _ = handle.eof(channel_id).await;
            let _ = handle.close(channel_id).await;
        });

        Ok(())
    }
}

/// Run git-receive-pack subprocess, then sync new objects to chain.
async fn run_receive_pack(
    repo_path: &std::path::Path,
    repo_name: &str,
    channel_id: ChannelId,
    handle: &russh::server::Handle,
    cache: &RepoCache,
) -> Result<()> {
    let mut child = tokio::process::Command::new("git")
        .args(["receive-pack", repo_path.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let mut stdout = child.stdout.take().unwrap();
    let handle_out = handle.clone();
    tokio::spawn(async move {
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

    let mut stderr = child.stderr.take().unwrap();
    let handle_err = handle.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 8192];
        loop {
            match stderr.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let _ = handle_err
                        .extended_data(channel_id, 1, CryptoVec::from(&buf[..n]))
                        .await;
                }
                Err(_) => break,
            }
        }
    });

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("git receive-pack exited with {}", status);
    }

    tracing::info!(repo = repo_name, "receive-pack complete, syncing to chain");
    let result = push_handler::sync_to_chain(repo_path, repo_name, cache.contract()).await?;

    match result {
        push_handler::PushSyncResult::Pushed { objects_uploaded } => {
            let msg = format!(
                "remote: Synced to Vastrum chain ({} objects uploaded)\r\n",
                objects_uploaded
            );
            let _ = handle
                .extended_data(channel_id, 1, CryptoVec::from(msg.as_bytes()))
                .await;
        }
        push_handler::PushSyncResult::AlreadyUpToDate => {
            let _ = handle
                .extended_data(
                    channel_id,
                    1,
                    CryptoVec::from(b"remote: Already up to date on chain\r\n" as &[u8]),
                )
                .await;
        }
    }

    Ok(())
}

/// Load an existing SSH host key or generate a new ed25519 key.
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
