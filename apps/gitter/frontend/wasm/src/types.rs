use vastrum_git_lib::universal::{differ::FileDiff, directory_explorer::ExplorerEntry};
use serde::Serialize;
use tsify::Tsify;

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct GitRepository {
    pub name: String,
    pub description: String,
    pub owner: String, // Ed25519PublicKey as hex string
    pub head_commit_hash: String,
    pub ssh_key_fingerprint: Option<String>, // "SHA256:<base64-no-pad>" when set
}

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub base_repo: String,
    pub base_branch: String,
    pub head_repo: String,
    pub head_branch: String,
    pub reply_count: u64,
    pub is_open: bool,
    pub is_merged: bool,
    pub from: String,
    pub created_at: u64,
}

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct PullRequestReply {
    pub content: String,
    pub timestamp: u64,
    pub from: String,
}

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct Issue {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub timestamp: u64,
    pub from: String,
    pub reply_count: u64,
}

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct IssueReply {
    pub content: String,
    pub timestamp: u64,
    pub from: String,
}

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct Discussion {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub timestamp: u64,
    pub from: String,
    pub reply_count: u64,
}

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct DiscussionReply {
    pub content: String,
    pub timestamp: u64,
    pub from: String,
}

#[derive(Serialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi)]
pub struct RepoCounts {
    pub issue_count: u64,
    pub pr_count: u64,
    pub discussion_count: u64,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct GetRepoDetail {
    pub git_repo: GitRepository,
    pub head_commit_author_name: String,
    pub head_commit_message: String,
    pub head_commit_hash: String,
    pub readme_contents: String,
    pub top_level_files: Vec<ExplorerEntry>,
    pub issue_count: u64,
    pub pr_count: u64,
    pub discussion_count: u64,
    pub is_owner: bool,
    pub branches: Vec<String>,
    pub current_branch: String,
    pub default_branch: String,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub enum FrontendMergability {
    CanMerge,
    CannotMergeConflict,
    AlreadyUpToDate,
    AlreadyMerged,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct FrontendCommit {
    pub author_name: String,
    pub author_timestamp: u64,
    pub message: String,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct GetPullRequestDetail {
    pub pull_request: PullRequest,
    pub commits_to_merge: Vec<FrontendCommit>,
    pub file_changes: Vec<FileDiff>,
    pub frontend_mergability: FrontendMergability,
    pub is_owner: bool,
}
