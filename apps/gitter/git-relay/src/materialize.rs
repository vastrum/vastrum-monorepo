use anyhow::{Context, Result, anyhow, bail};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use futures::StreamExt;
use gix::{
    ObjectId, ThreadSafeRepository, create,
    create::Kind,
    objs::tree::EntryKind,
    open,
    refs::{
        Target,
        transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog},
    },
};
use gix_object::{CommitRef, ObjectRef, TagRef, TreeRef};
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::universal::utils::{GitContext, sha1_to_oid};

/// Create and populate a bare repo from on-chain state.
/// Downloads all branches and writes each as `refs/heads/{branch}`.
/// Sets HEAD symbolic ref to the default branch.
pub async fn materialize_bare_repo(
    bare_path: &Path,
    repo_name: &str,
    contract: &ContractAbiClient,
) -> Result<()> {
    let state = contract.state().await;
    let repo_info = state
        .repo_store
        .get(&repo_name.to_string())
        .await
        .with_context(|| format!("repo '{}' not found on-chain", repo_name))?;

    // Create bare repo
    let repo = ThreadSafeRepository::init_opts(
        bare_path,
        Kind::Bare,
        create::Options::default(),
        open::Options::default(),
    )?
    .to_thread_local();

    let ctx = Arc::new(GitContext::new(state.git_object_store));
    let objects_dir: Arc<PathBuf> = Arc::new(bare_path.join("objects"));

    // Download each branch and write its ref.

    for (branch_name, hash) in &repo_info.branches {
        let oid = sha1_to_oid(hash);
        download_commits_parallel(objects_dir.clone(), oid, ctx.clone()).await?;

        let ref_name = format!("refs/heads/{}", branch_name);
        repo.edit_reference(RefEdit {
            change: Change::Update {
                log: LogChange {
                    mode: RefLog::AndReference,
                    force_create_reflog: false,
                    message: "materialize from chain".into(),
                },
                expected: PreviousValue::Any,
                new: Target::Object(oid),
            },
            name: ref_name.try_into().unwrap(),
            deref: false,
        })?;
    }

    // Set HEAD to default branch (even if the branch doesn't exist yet — unborn HEAD
    // is fine for git-receive-pack, it creates the branch on first push).
    let head_target = format!("refs/heads/{}", repo_info.default_branch);
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: "set HEAD to default branch".into(),
            },
            expected: PreviousValue::Any,
            new: Target::Symbolic(head_target.try_into().unwrap()),
        },
        name: "HEAD".try_into().unwrap(),
        deref: false,
    })?;

    Ok(())
}

async fn download_commits_parallel(
    objects_dir: Arc<PathBuf>,
    root: ObjectId,
    ctx: Arc<GitContext>,
) -> Result<()> {
    const CONCURRENCY: usize = 64;

    let visited: Arc<Mutex<HashSet<ObjectId>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut frontier: Vec<ObjectId> = vec![root];

    while !frontier.is_empty() {
        let level = std::mem::take(&mut frontier);
        let results: Vec<Result<Vec<ObjectId>>> = futures::stream::iter(level)
            .map(|oid| {
                let ctx = ctx.clone();
                let visited = visited.clone();
                let objects_dir = objects_dir.clone();
                async move {
                    {
                        let mut v = visited.lock().await;
                        if !v.insert(oid) {
                            return Ok(Vec::new());
                        }
                    }
                    // Fetch raw loose bytes from chain (one RPC).
                    let raw = ctx
                        .object_raw_bytes(oid)
                        .await
                        .map_err(|e| anyhow!("fetch {}: {}", oid, e))?;
                    let children = extract_children(&raw)?;

                    if !loose_object_exists(&objects_dir, oid) {
                        let od = objects_dir.clone();
                        tokio::task::spawn_blocking(move || write_loose_object_raw(&od, oid, &raw))
                            .await
                            .map_err(|e| anyhow!("spawn_blocking join: {}", e))??;
                    }

                    Ok(children)
                }
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;

        for r in results {
            frontier.extend(r?);
        }
    }
    Ok(())
}

fn extract_children(raw: &[u8]) -> Result<Vec<ObjectId>> {
    let obj_ref = ObjectRef::from_loose(raw).map_err(|e| anyhow!("loose decode: {}", e))?;
    let decode_oid = |hex: &[u8]| -> Result<ObjectId> {
        ObjectId::from_hex(hex).map_err(|e| anyhow!("oid decode: {}", e))
    };
    match obj_ref {
        ObjectRef::Commit(CommitRef { tree, parents, .. }) => {
            let mut out = Vec::with_capacity(1 + parents.len());
            out.push(decode_oid(tree)?);
            for p in parents.iter() {
                out.push(decode_oid(p)?);
            }
            Ok(out)
        }
        ObjectRef::Tree(TreeRef { entries }) => {
            let mut out = Vec::with_capacity(entries.len());
            for entry in entries {
                if entry.mode.kind() == EntryKind::Commit {
                    bail!("submodules are not supported");
                }
                out.push(entry.oid.to_owned());
            }
            Ok(out)
        }
        ObjectRef::Tag(TagRef { target, .. }) => Ok(vec![decode_oid(target)?]),
        ObjectRef::Blob(_) => Ok(Vec::new()),
    }
}

fn loose_object_path(objects_dir: &Path, oid: ObjectId) -> PathBuf {
    let hex = oid.to_string();
    objects_dir.join(&hex[..2]).join(&hex[2..])
}

fn loose_object_exists(objects_dir: &Path, oid: ObjectId) -> bool {
    loose_object_path(objects_dir, oid).exists()
}

fn write_loose_object_raw(objects_dir: &Path, expected_oid: ObjectId, raw: &[u8]) -> Result<()> {
    let digest = Sha1::digest(raw);
    let actual =
        ObjectId::try_from(digest.as_slice()).map_err(|e| anyhow!("hash to oid: {}", e))?;
    if actual != expected_oid {
        bail!("object hash mismatch: expected {}, got {}", expected_oid, actual);
    }

    let path = loose_object_path(objects_dir, expected_oid);
    if path.exists() {
        return Ok(());
    }

    let mut enc = ZlibEncoder::new(Vec::with_capacity(raw.len()), Compression::default());
    enc.write_all(raw)?;
    let compressed = enc.finish()?;

    let dir = path.parent().expect("loose object path has parent");
    fs::create_dir_all(dir)?;
    let file_name = path.file_name().expect("loose object path has file name");
    let tmp = dir.join(format!(".{}.tmp", file_name.to_string_lossy()));
    fs::write(&tmp, &compressed)?;
    match fs::rename(&tmp, &path) {
        Ok(()) => Ok(()),
        Err(_) if path.exists() => {
            let _ = fs::remove_file(&tmp);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
