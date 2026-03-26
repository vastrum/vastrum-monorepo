pub struct TestRepo {
    _tempdir: tempfile::TempDir,
    repo: Repository,
    time: Time,
}

impl TestRepo {
    pub fn path_str(&self) -> &str {
        let path = self._tempdir.path().to_str().unwrap();
        return path;
    }

    pub fn head_id(&self) -> ObjectId {
        let head = self.repo.head().unwrap().id().unwrap().detach();
        return head;
    }

    pub fn blob_id(&self, path: &str) -> ObjectId {
        let head = self.repo.head().unwrap().peel_to_commit().unwrap();
        let tree = head.tree().unwrap();
        let entry = tree.lookup_entry_by_path(path).unwrap().unwrap();
        let blob_id = entry.object_id();
        return blob_id;
    }

    pub fn add_commit(&self, files: &[(&str, &[u8])]) -> ObjectId {
        let tree_id = Self::build_tree_recursive(&self.repo, files, &[]);
        let parent = self.repo.head().unwrap().id().unwrap().detach();
        let commit_id = self.write_commit(tree_id, vec![parent]);
        self.update_head(commit_id);
        return commit_id;
    }

    fn update_head(&self, commit_id: ObjectId) {
        self.repo
            .edit_reference(RefEdit {
                change: Change::Update {
                    log: LogChange {
                        mode: RefLog::AndReference,
                        force_create_reflog: false,
                        message: "update".into(),
                    },
                    expected: PreviousValue::Any,
                    new: Target::Object(commit_id),
                },
                name: "HEAD".try_into().unwrap(),
                deref: false,
            })
            .unwrap();
    }

    fn write_commit(&self, tree_id: ObjectId, parents: Vec<ObjectId>) -> ObjectId {
        let sig = Signature {
            name: "natsec".into(),
            email: "test@test.com".into(),
            time: self.time.clone(),
        };
        let commit = Commit {
            tree: tree_id,
            parents: parents.into(),
            author: sig.clone(),
            committer: sig,
            encoding: None,
            message: "Initial Commit".into(),
            extra_headers: vec![],
        };
        let object_id = self.repo.write_object(&commit).unwrap().detach();
        return object_id;
    }

    fn build_tree_recursive(
        repo: &Repository,
        files: &[(&str, &[u8])],
        empty_dirs: &[&str],
    ) -> ObjectId {
        let mut here_files: Vec<(&str, &[u8])> = Vec::new();
        let mut subdirs: HashMap<&str, Vec<(&str, &[u8])>> = HashMap::new();
        let mut here_empty_dirs: Vec<&str> = Vec::new();
        let mut sub_empty_dirs: HashMap<&str, Vec<&str>> = HashMap::new();

        for &(path, content) in files {
            if let Some((dir, rest)) = path.split_once('/') {
                subdirs.entry(dir).or_default().push((rest, content));
            } else {
                here_files.push((path, content));
            }
        }

        for &dir_path in empty_dirs {
            if let Some((dir, rest)) = dir_path.split_once('/') {
                if rest.is_empty() {
                    here_empty_dirs.push(dir);
                } else {
                    sub_empty_dirs.entry(dir).or_default().push(rest);
                }
            } else {
                here_empty_dirs.push(dir_path);
            }
        }

        let mut entries = Vec::new();

        for (name, content) in here_files {
            let blob = Blob { data: content.to_vec() };
            let blob_id = repo.write_object(&blob).unwrap().detach();
            entries.push(Entry {
                mode: EntryKind::Blob.into(),
                filename: name.into(),
                oid: blob_id,
            });
        }

        for (dir_name, sub_files) in &subdirs {
            let sub_empty: Vec<&str> = sub_empty_dirs.remove(*dir_name).unwrap_or_default();
            let subtree_id = Self::build_tree_recursive(repo, sub_files, &sub_empty);
            entries.push(Entry {
                mode: EntryKind::Tree.into(),
                filename: (*dir_name).into(),
                oid: subtree_id,
            });
        }

        for dir_name in here_empty_dirs {
            if !subdirs.contains_key(dir_name) {
                let sub_empty: Vec<&str> = sub_empty_dirs.remove(dir_name).unwrap_or_default();
                let subtree_id = if sub_empty.is_empty() {
                    let empty_tree = Tree { entries: vec![] };
                    repo.write_object(&empty_tree).unwrap().detach()
                } else {
                    Self::build_tree_recursive(repo, &[], &sub_empty)
                };
                entries.push(Entry {
                    mode: EntryKind::Tree.into(),
                    filename: dir_name.into(),
                    oid: subtree_id,
                });
            }
        }

        entries.sort_by(|a, b| {
            let a_key =
                if a.mode.is_tree() { format!("{}/", a.filename) } else { a.filename.to_string() };
            let b_key =
                if b.mode.is_tree() { format!("{}/", b.filename) } else { b.filename.to_string() };
            a_key.cmp(&b_key)
        });

        let tree = Tree { entries };
        let object_id = repo.write_object(&tree).unwrap().detach();
        return object_id;
    }
}

pub struct TestRepoBuilder {
    files: Vec<(String, Vec<u8>)>,
    empty_dirs: Vec<String>,
    time: Time,
}

impl TestRepoBuilder {
    pub fn new() -> Self {
        return Self { files: Vec::new(), empty_dirs: Vec::new(), time: Time::new(0, 0) };
    }

    pub fn file(mut self, path: &str, content: &[u8]) -> Self {
        self.files.push((path.to_string(), content.to_vec()));
        return self;
    }

    pub fn dir(mut self, path: &str) -> Self {
        self.empty_dirs.push(path.to_string());
        return self;
    }

    pub fn time(mut self, seconds: i64) -> Self {
        self.time = Time::new(seconds, 0);
        return self;
    }

    pub fn build(self) -> TestRepo {
        let time = self.time;
        let tempdir = tempfile::TempDir::new().unwrap();
        let mut repo = gix::init(tempdir.path()).unwrap();
        let _ = repo.committer_or_set_generic_fallback();
        let test_repo = TestRepo { _tempdir: tempdir, repo, time };

        let file_refs: Vec<(&str, &[u8])> =
            self.files.iter().map(|(s, v)| (s.as_str(), v.as_slice())).collect();
        let dir_refs: Vec<&str> = self.empty_dirs.iter().map(|s| s.as_str()).collect();

        let tree_id = TestRepo::build_tree_recursive(&test_repo.repo, &file_refs, &dir_refs);
        let commit_id = test_repo.write_commit(tree_id, vec![]);
        test_repo.update_head(commit_id);

        return test_repo;
    }
}

pub struct TestContext {
    pub contract: ContractAbiClient,
    pub account_key: ed25519::PrivateKey,
}

impl TestContext {
    pub async fn new() -> Self {
        vastrum_native_lib::test_support::ensure_localnet("../contract", "../contract/out");
        let client = ContractAbiClient::deploy("../contract/out/contract.wasm", vec![]).await;
        let account_key = ed25519::PrivateKey::from_seed(111);
        let ctx = Self { contract: client.with_account_key(account_key.clone()), account_key };
        return ctx;
    }
}

use crate::ContractAbiClient;
use gix::{
    ObjectId, Repository,
    actor::Signature,
    date::Time,
    objs::{
        Blob, Commit, Tree,
        tree::{Entry, EntryKind},
    },
    refs::{
        Target,
        transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog},
    },
};
use vastrum_shared_types::crypto::ed25519;
use std::collections::HashMap;
