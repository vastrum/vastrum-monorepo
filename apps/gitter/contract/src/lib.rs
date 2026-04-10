#[contract_state]
struct Contract {
    repo_store: KvMap<String, GitRepository>,
    all_repos: KvVecBTree<u64, GitRepository>,
    forks_store: KvMap<ForksKey, Vec<String>>,
    git_object_store: KvMap<Sha1Hash, Vec<u8>>,
    relay_key: Option<Ed25519PublicKey>,
}

#[contract_methods]
impl Contract {
    /// Stores a git object in the contract KV store (commit, blob, tree)
    /// Contract hashes it to verify objectID
    pub fn upload_git_object(&mut self, data: Vec<u8>) {
        let hash = Sha1Hash(Sha1::digest(&data).into());
        self.git_object_store.set(&hash, data);
    }
    #[authenticated]
    pub fn create_repository(&mut self, name: String, description: String) {
        if name.len() > MAX_NAME_LEN || description.len() > MAX_DESCRIPTION_LEN {
            return;
        }

        let name_already_used = self.repo_store.contains(&name);
        if name_already_used {
            return;
        }

        let repo = GitRepository {
            name: name.clone(),
            description,
            owner: message_sender(),
            head_commit_hash: None,
            ssh_key_fingerprint: None,
            issues: KvVecBTree::default(),
            pull_requests: KvVecBTree::default(),
            discussions: KvVecBTree::default(),
        };

        self.repo_store.set(&name, repo.clone());

        let timestamp = block_time();
        self.all_repos.push(timestamp, repo);
    }

    #[authenticated]
    pub fn set_head_commit(&mut self, name: String, commit_hash: Sha1Hash) {
        let mut repo = self.repo_store.get(&name).unwrap();

        if repo.owner != message_sender() {
            panic!("not the repo owner");
        }

        repo.head_commit_hash = Some(commit_hash);
        self.repo_store.set(&name, repo);
    }

    #[authenticated]
    pub fn fork_repository(&mut self, new_repo_name: String, repo_to_fork_name: String) {
        if new_repo_name.len() > MAX_NAME_LEN {
            return;
        }
        let name_occupied = self.repo_store.contains(&new_repo_name);
        if name_occupied {
            return;
        }
        let old_repo = self.repo_store.get(&repo_to_fork_name).unwrap();

        let repo = GitRepository {
            name: new_repo_name.clone(),
            description: old_repo.description,
            owner: message_sender(),
            head_commit_hash: old_repo.head_commit_hash,
            ssh_key_fingerprint: None,
            issues: KvVecBTree::default(),
            pull_requests: KvVecBTree::default(),
            discussions: KvVecBTree::default(),
        };

        self.repo_store.set(&new_repo_name, repo.clone());

        let timestamp = block_time();
        self.all_repos.push(timestamp, repo);

        let forks_key = ForksKey { repo_name: repo_to_fork_name, from: message_sender() };
        let mut forks = self.forks_store.get(&forks_key).unwrap_or(vec![]);
        forks.push(new_repo_name);
        self.forks_store.set(&forks_key, forks);
    }

    #[authenticated]
    pub fn create_pull_request(
        &mut self,
        to_repo: String,
        merging_repo: String,
        title: String,
        description: String,
    ) {
        if title.len() > MAX_TITLE_LEN || description.len() > MAX_CONTENT_LEN {
            return;
        }
        let repo = self.repo_store.get(&to_repo).unwrap();

        let timestamp = block_time();
        let id = repo.pull_requests.next_id();
        let pull_request = PullRequest {
            id,
            title,
            description,
            merging_repo,
            reply_count: 0,
            replies: KvVecBTree::default(),
            is_open: true,
            is_merged: false,
            from: message_sender(),
            created_at: timestamp,
        };

        repo.pull_requests.push(timestamp, pull_request);
    }

    #[authenticated]
    pub fn reply_to_pull_request(&mut self, content: String, repo_name: String, id: u64) {
        if content.len() > MAX_CONTENT_LEN {
            return;
        }
        let repo = self.repo_store.get(&repo_name).unwrap();
        let reply = PullRequestReply { content, timestamp: block_time(), from: message_sender() };

        let timestamp = block_time();

        let mut pull_request = repo.pull_requests.get(id).unwrap();
        pull_request.reply_count += 1;
        pull_request.replies.push(timestamp, reply);

        repo.pull_requests.update(id, timestamp, pull_request);
    }

    #[authenticated]
    pub fn close_pull_request(&mut self, repo_name: String, id: u64) {
        let repo = self.repo_store.get(&repo_name).unwrap();
        if repo.owner != message_sender() {
            panic!("not the repo owner");
        }

        let mut pull_request = repo.pull_requests.get(id).unwrap();

        pull_request.is_open = false;

        repo.pull_requests.update(id, block_time(), pull_request);
    }

    #[authenticated]
    pub fn mark_pull_request_merged(&mut self, repo_name: String, id: u64) {
        let repo = self.repo_store.get(&repo_name).unwrap();
        if repo.owner != message_sender() {
            panic!("not the repo owner");
        }

        let mut pull_request = repo.pull_requests.get(id).unwrap();

        pull_request.is_merged = true;
        pull_request.is_open = false;

        repo.pull_requests.update(id, block_time(), pull_request);
    }

    #[authenticated]
    pub fn create_issue(&mut self, title: String, description: String, repo_name: String) {
        if title.len() > MAX_TITLE_LEN || description.len() > MAX_CONTENT_LEN {
            return;
        }
        let repo = self.repo_store.get(&repo_name).unwrap();
        let timestamp = block_time();

        let id = repo.issues.next_id();
        let issue = Issue {
            id,
            title,
            description,
            timestamp,
            from: message_sender(),
            reply_count: 0,
            replies: KvVecBTree::default(),
        };
        repo.issues.push(timestamp, issue);
    }

    #[authenticated]
    pub fn reply_to_issue(&mut self, content: String, repo_name: String, id: u64) {
        if content.len() > MAX_CONTENT_LEN {
            return;
        }
        let repo = self.repo_store.get(&repo_name).unwrap();

        let timestamp = block_time();

        let reply = IssueReply { content, timestamp, from: message_sender() };

        let mut issue = repo.issues.get(id).unwrap();

        issue.reply_count += 1;
        issue.replies.push(timestamp, reply);

        repo.issues.update(id, timestamp, issue);
    }

    #[authenticated]
    pub fn create_discussion(&mut self, title: String, description: String, repo_name: String) {
        if title.len() > MAX_TITLE_LEN || description.len() > MAX_CONTENT_LEN {
            return;
        }
        let repo = self.repo_store.get(&repo_name).unwrap();
        let timestamp = block_time();

        let id = repo.discussions.next_id();
        let discussion = Discussion {
            id,
            title,
            description,
            timestamp,
            from: message_sender(),
            reply_count: 0,
            replies: KvVecBTree::default(),
        };
        repo.discussions.push(timestamp, discussion);
    }

    #[authenticated]
    pub fn reply_to_discussion(&mut self, content: String, repo_name: String, id: u64) {
        if content.len() > MAX_CONTENT_LEN {
            return;
        }
        let repo = self.repo_store.get(&repo_name).unwrap();

        let timestamp = block_time();

        let reply = DiscussionReply { content, timestamp, from: message_sender() };

        let mut discussion = repo.discussions.get(id).unwrap();

        discussion.reply_count += 1;
        discussion.replies.push(timestamp, reply);

        repo.discussions.update(id, timestamp, discussion);
    }
    #[authenticated]
    pub fn set_ssh_key_fingerprint(&mut self, repo_name: String, fingerprint: SshKeyFingerprint) {
        let mut repo = self.repo_store.get(&repo_name).unwrap();
        if repo.owner != message_sender() {
            panic!("not the repo owner");
        }
        repo.ssh_key_fingerprint = Some(fingerprint);
        self.repo_store.set(&repo_name, repo);
    }

    #[authenticated]
    pub fn relay_set_head_commit(&mut self, repo_name: String, commit_hash: Sha1Hash) {
        if message_sender() != self.relay_key.unwrap() {
            panic!("not the relay");
        }
        let mut repo = self.repo_store.get(&repo_name).unwrap();
        repo.head_commit_hash = Some(commit_hash);
        self.repo_store.set(&repo_name, repo);
    }

    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>, relay_key: Ed25519PublicKey) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        let mut state = Self::default();
        state.relay_key = Some(relay_key);
        return state;
    }
}

const MAX_NAME_LEN: usize = 100;
const MAX_TITLE_LEN: usize = 200;
const MAX_DESCRIPTION_LEN: usize = 512;
const MAX_CONTENT_LEN: usize = 16000;

#[contract_type]
struct GitRepository {
    name: String,
    description: String,
    owner: Ed25519PublicKey,
    head_commit_hash: Option<Sha1Hash>,
    ssh_key_fingerprint: Option<SshKeyFingerprint>,
    issues: KvVecBTree<u64, Issue>,
    pull_requests: KvVecBTree<u64, PullRequest>,
    discussions: KvVecBTree<u64, Discussion>,
}

#[contract_type]
struct PullRequest {
    id: u64,
    title: String,
    description: String,
    merging_repo: String,
    reply_count: u64,
    replies: KvVecBTree<u64, PullRequestReply>,
    is_open: bool,
    is_merged: bool,
    from: Ed25519PublicKey,
    created_at: u64,
}

#[contract_type]
struct PullRequestReply {
    content: String,
    timestamp: u64,
    from: Ed25519PublicKey,
}

#[contract_type]
struct Issue {
    id: u64,
    title: String,
    description: String,
    timestamp: u64,
    from: Ed25519PublicKey,
    reply_count: u64,
    replies: KvVecBTree<u64, IssueReply>,
}

#[contract_type]
struct IssueReply {
    content: String,
    timestamp: u64,
    from: Ed25519PublicKey,
}

#[contract_type]
struct Discussion {
    id: u64,
    title: String,
    description: String,
    timestamp: u64,
    from: Ed25519PublicKey,
    reply_count: u64,
    replies: KvVecBTree<u64, DiscussionReply>,
}

#[contract_type]
struct DiscussionReply {
    content: String,
    timestamp: u64,
    from: Ed25519PublicKey,
}

#[contract_type]
struct ForksKey {
    repo_name: String,
    from: Ed25519PublicKey,
}
#[derive(Copy, PartialEq)]
#[contract_type]
struct Sha1Hash([u8; 20]);

#[derive(Copy, PartialEq)]
#[contract_type]
struct SshKeyFingerprint([u8; 32]);

use sha1::{Digest, Sha1};
use vastrum_contract_macros::{
    authenticated, constructor, contract_methods, contract_state, contract_type,
};
use vastrum_runtime_lib::{
    Ed25519PublicKey, KvMap, KvVecBTree,
    runtime::{block_time, message_sender},
};
