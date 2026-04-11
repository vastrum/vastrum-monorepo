#[wasm_bindgen]
pub async fn create_issue(title: String, description: String, repo_name: String) -> String {
    let gitter = new_client();
    let sent_tx = gitter.create_issue(title, description, repo_name).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn reply_to_issue(content: String, repo_name: String, issue_id: u64) -> String {
    let gitter = new_client();
    let sent_tx = gitter.reply_to_issue(content, repo_name, issue_id).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn create_discussion(title: String, description: String, repo_name: String) -> String {
    let gitter = new_client();
    let sent_tx = gitter.create_discussion(title, description, repo_name).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn reply_to_discussion(content: String, repo_name: String, discussion_id: u64) -> String {
    let gitter = new_client();
    let sent_tx = gitter.reply_to_discussion(content, repo_name, discussion_id).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn create_repo(name: String, description: String) -> String {
    let gitter = new_client();
    let sent_tx = gitter.create_repository(name, description).await;
    sent_tx.tx_hash().to_string()
}

/// Accepts an SSH public key string (e.g. "ssh-ed25519 AAAA... user@host")
/// and registers its SHA256 fingerprint for the repo.
/// Caller must validate key format first (see utils/sshKey.ts).
#[wasm_bindgen]
pub async fn set_ssh_key_fingerprint(repo_name: String, ssh_public_key: String) -> String {
    let fingerprint = parse_ssh_key_fingerprint(&ssh_public_key);
    let gitter = new_client();
    let sent_tx = gitter.set_ssh_key_fingerprint(repo_name, fingerprint).await;
    sent_tx.tx_hash().to_string()
}

fn parse_ssh_key_fingerprint(ssh_public_key: &str) -> vastrum_git_lib::SshKeyFingerprint {
    // SSH public key format: "type base64-data [comment]"
    let parts: Vec<&str> = ssh_public_key.trim().split_whitespace().collect();
    assert!(parts.len() >= 2, "invalid SSH public key format");
    use base64::Engine;
    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(parts[1])
        .expect("invalid base64 in SSH key");
    let hash = sha256_hash(&key_bytes);
    let fingerprint = vastrum_git_lib::SshKeyFingerprint(hash.to_bytes());
    return fingerprint;
}

#[wasm_bindgen]
pub async fn fork_repo(new_repo_name: String, repo_to_fork_name: String) -> String {
    let gitter = new_client();
    let sent_tx = gitter.fork_repository(new_repo_name, repo_to_fork_name).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn get_forks_by_me_of_this_repo(repo_name: String) -> Vec<String> {
    let gitter = new_client();
    let pub_key = get_pub_key().await;

    let state = gitter.state().await;
    let forks_key = vastrum_git_lib::ForksKey { repo_name, from: pub_key };
    let all_forks_by_me = state.forks_store.get(&forks_key).await;
    all_forks_by_me.unwrap_or_default()
}

#[wasm_bindgen]
pub async fn get_default_fork_name(repo_name: String) -> String {
    let gitter = new_client();
    let pub_key = get_pub_key().await;
    let key_prefix = &pub_key.to_string()[..8];

    let state = gitter.state().await;
    let forks_key = vastrum_git_lib::ForksKey { repo_name: repo_name.clone(), from: pub_key };
    let existing_forks = state.forks_store.get(&forks_key).await.unwrap_or_default();
    let n = existing_forks.len() + 1;

    return format!("{}-fork-{}-{}", repo_name, key_prefix, n);
}

#[wasm_bindgen]
pub async fn create_pull_request(
    base_repo: String,
    base_branch: String,
    head_repo: String,
    head_branch: String,
    title: String,
    description: String,
) -> String {
    let gitter = new_client();
    let sent_tx = gitter
        .create_pull_request(base_repo, base_branch, head_repo, head_branch, title, description)
        .await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn reply_to_pull_request(
    content: String,
    repo_name: String,
    pull_request_id: u64,
) -> String {
    let gitter = new_client();
    let sent_tx = gitter.reply_to_pull_request(content, repo_name, pull_request_id).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn close_pull_request(repo_name: String, pull_request_id: u64) -> String {
    let gitter = new_client();
    let sent_tx = gitter.close_pull_request(repo_name, pull_request_id).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn merge_pull_request(repo_name: String, pull_request_id: u64) -> String {
    let gitter = new_client();
    // Look up PR to get branches
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let pr = repo.pull_requests.get(pull_request_id).await.unwrap();
    merge_repos(
        pr.base_repo.clone(),
        pr.base_branch.clone(),
        pr.head_repo.clone(),
        pr.head_branch.clone(),
        &gitter,
        MergeMode::Live,
    )
    .await
    .unwrap();
    let sent_tx = gitter.mark_pull_request_merged(repo_name, pull_request_id).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn get_all_repos() -> Vec<GitRepository> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repos = state.all_repos.get_descending_entries(50, 0).await;
    repos
        .iter()
        .map(|r| {
            let head = r
                .branches
                .get(&r.default_branch)
                .map(|h| sha1_to_oid(h).to_string())
                .unwrap_or_default();
            convert_git_repository(r, &head)
        })
        .collect()
}

#[wasm_bindgen]
pub async fn get_repo_issues(repo_name: String, limit: usize, offset: usize) -> Vec<Issue> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let issues = repo.issues.get_descending_entries(limit, offset).await;
    issues.iter().map(convert_issue).collect()
}

#[wasm_bindgen]
pub async fn get_repo_pull_requests(
    repo_name: String,
    limit: usize,
    offset: usize,
) -> Vec<PullRequest> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let prs = repo.pull_requests.get_descending_entries(limit, offset).await;
    prs.iter().map(convert_pull_request).collect()
}

#[wasm_bindgen]
pub async fn get_repo_discussions(
    repo_name: String,
    limit: usize,
    offset: usize,
) -> Vec<Discussion> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let discussions = repo.discussions.get_descending_entries(limit, offset).await;
    discussions.iter().map(convert_discussion).collect()
}

#[wasm_bindgen]
pub async fn get_discussion(repo_name: String, id: u64) -> Option<Discussion> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    repo.discussions.get(id).await.map(|d| convert_discussion(&d))
}

#[wasm_bindgen]
pub async fn get_issue(repo_name: String, id: u64) -> Option<Issue> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    repo.issues.get(id).await.map(|i| convert_issue(&i))
}

#[wasm_bindgen]
pub async fn get_pull_request(repo_name: String, id: u64) -> Option<PullRequest> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    repo.pull_requests.get(id).await.map(|p| convert_pull_request(&p))
}

#[wasm_bindgen]
pub async fn get_issue_replies(
    repo_name: String,
    issue_id: u64,
    limit: usize,
    offset: usize,
) -> Vec<IssueReply> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let issue = repo.issues.get(issue_id).await.unwrap();
    let replies = issue.replies.get_ascending_entries(limit, offset).await;
    replies.iter().map(convert_issue_reply).collect()
}

#[wasm_bindgen]
pub async fn get_pull_request_replies(
    repo_name: String,
    pr_id: u64,
    limit: usize,
    offset: usize,
) -> Vec<PullRequestReply> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let pr = repo.pull_requests.get(pr_id).await.unwrap();
    let replies = pr.replies.get_ascending_entries(limit, offset).await;
    replies.iter().map(convert_pr_reply).collect()
}

#[wasm_bindgen]
pub async fn get_discussion_replies(
    repo_name: String,
    discussion_id: u64,
    limit: usize,
    offset: usize,
) -> Vec<DiscussionReply> {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let discussion = repo.discussions.get(discussion_id).await.unwrap();
    let replies = discussion.replies.get_ascending_entries(limit, offset).await;
    replies.iter().map(convert_discussion_reply).collect()
}

#[wasm_bindgen]
pub async fn get_repo_counts(repo_name: String) -> RepoCounts {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let (issue_count, pr_count, discussion_count) = futures::join!(
        repo.issues.length(),
        repo.pull_requests.length(),
        repo.discussions.length(),
    );
    RepoCounts { issue_count, pr_count, discussion_count }
}

#[wasm_bindgen]
pub async fn get_repo_page_data(repo_name: String, branch: Option<String>) -> GetRepoDetail {
    let gitter = new_client();
    let state = gitter.state().await;
    let repo = state.repo_store.get(&repo_name).await.unwrap();
    let ctx = GitContext::new(state.git_object_store.clone());

    // Resolve which branch to show
    let branch_names: Vec<String> = repo.branches.keys().cloned().collect();
    let current_branch = branch.unwrap_or_else(|| {
        if repo.branches.contains_key(&repo.default_branch) {
            repo.default_branch.clone()
        } else {
            // Default branch missing (stale or empty) — pick first available
            branch_names.first().cloned().unwrap_or_default()
        }
    });

    let current_commit = repo.branches.get(&current_branch).cloned();

    let (
        head_commit_author_name,
        head_commit_message,
        head_commit_hash,
        top_level_files,
        readme_text,
        issue_count,
        pr_count,
        discussion_count,
    ) = if let Some(hash) = current_commit {
        let head_oid = sha1_to_oid(&hash);
        let head_commit = ctx.read_commit(head_oid).await.unwrap();

        let (tree_result, ic, pc, dc) = futures::join!(
            get_files_for_tree(head_commit.tree, &ctx),
            repo.issues.length(),
            repo.pull_requests.length(),
            repo.discussions.length(),
        );

        let top_files = tree_result.unwrap();
        let mut readme = String::new();
        if let Some(readme_entry) = top_files.iter().find(|e| e.name == "README.md") {
            let oid = ObjectId::from_str(&readme_entry.oid).unwrap();
            let data = get_file_data(oid, &ctx).await.unwrap_or_default();
            readme = String::from_utf8(data).unwrap_or_default();
        }
        (
            head_commit.author.name.to_string(),
            head_commit.message.to_string(),
            head_oid.to_string(),
            top_files,
            readme,
            ic,
            pc,
            dc,
        )
    } else {
        let (ic, pc, dc) = futures::join!(
            repo.issues.length(),
            repo.pull_requests.length(),
            repo.discussions.length(),
        );
        (String::new(), String::new(), String::new(), Vec::new(), String::new(), ic, pc, dc)
    };

    let pub_key = get_pub_key().await;
    let is_owner = pub_key == repo.owner;
    let default_branch = repo.default_branch.clone();
    let converted_repo = convert_git_repository(&repo, &head_commit_hash);
    GetRepoDetail {
        git_repo: converted_repo,
        head_commit_author_name,
        head_commit_message,
        head_commit_hash,
        readme_contents: readme_text,
        top_level_files,
        issue_count,
        pr_count,
        discussion_count,
        is_owner,
        branches: branch_names,
        current_branch,
        default_branch,
    }
}

#[wasm_bindgen]
pub async fn get_file(git_hash: String) -> String {
    let gitter = new_client();
    let ctx = GitContext::from_contract(&gitter).await;
    let oid = ObjectId::from_hex(git_hash.as_bytes()).unwrap();
    let data = get_file_data(oid, &ctx).await.unwrap();
    let file_is_binary = data.iter().take(8000).any(|&b| b == 0);
    if file_is_binary {
        return String::new();
    }
    String::from_utf8(data).unwrap_or_default()
}

#[wasm_bindgen]
pub async fn is_file_binary(git_hash: String) -> bool {
    let gitter = new_client();
    let ctx = GitContext::from_contract(&gitter).await;
    let oid = ObjectId::from_hex(git_hash.as_bytes()).unwrap();
    let data = get_file_data(oid, &ctx).await.unwrap();
    let file_is_binary = data.iter().take(8000).any(|&b| b == 0);
    return file_is_binary;
}

#[wasm_bindgen]
pub async fn get_directory_contents(tree_oid: String) -> Result<Vec<ExplorerEntry>, String> {
    if tree_oid.is_empty() {
        return Err("Invalid tree OID: empty string".to_string());
    }
    let gitter = new_client();
    let oid =
        ObjectId::from_hex(tree_oid.as_bytes()).map_err(|e| format!("Invalid tree OID: {}", e))?;
    let ctx = GitContext::from_contract(&gitter).await;
    Ok(get_files_for_tree(oid, &ctx).await.unwrap())
}

#[wasm_bindgen]
pub async fn get_pull_request_detail(repo_name: String, id: u64) -> GetPullRequestDetail {
    let gitter = new_client();
    let state = gitter.state().await;
    let ctx = GitContext::new(state.git_object_store.clone());

    let base_repo = state.repo_store.get(&repo_name).await.unwrap();
    let pull_request = base_repo.pull_requests.get(id).await.unwrap();

    // Look up head repo (can be same as base_repo for same-repo PRs)
    let head_repo_name = pull_request.head_repo.clone();
    let (head_repo_opt, pub_key) =
        futures::join!(state.repo_store.get(&head_repo_name), get_pub_key(),);
    let head_repo = head_repo_opt.unwrap();
    let is_owner = pub_key == base_repo.owner;

    let base_head = sha1_to_oid(base_repo.branches.get(&pull_request.base_branch).unwrap());
    let merge_head = sha1_to_oid(head_repo.branches.get(&pull_request.head_branch).unwrap());

    let frontend_status = if pull_request.is_merged {
        FrontendMergability::AlreadyMerged
    } else {
        let mergability =
            merge_branches(base_head, merge_head, &ctx, &gitter, MergeMode::Preview).await.unwrap();
        match mergability {
            MergeResult::FastForward(_) | MergeResult::Merged(_) => FrontendMergability::CanMerge,
            MergeResult::Conflict(_) | MergeResult::NoCommonAncestor => {
                FrontendMergability::CannotMergeConflict
            }
            MergeResult::AlreadyUpToDate => FrontendMergability::AlreadyUpToDate,
        }
    };

    let (diff_result, feature_commits) = futures::join!(
        diff_commits(base_head, merge_head, &ctx),
        ctx.get_feature_commits(base_head, merge_head),
    );

    let file_diffs = diff_result.unwrap();

    let mut frontend_commits = vec![];
    for commit in feature_commits.unwrap_or_default() {
        frontend_commits.push(FrontendCommit {
            author_name: commit.author.name.to_string(),
            author_timestamp: commit.author.time.seconds as u64,
            message: commit.message.to_string(),
        });
    }

    let converted_pr = convert_pull_request(&pull_request);

    GetPullRequestDetail {
        pull_request: converted_pr,
        commits_to_merge: frontend_commits,
        file_changes: file_diffs.files,
        frontend_mergability: frontend_status,
        is_owner,
    }
}

mod converters;
mod helpers;
mod types;

use converters::*;
use gix_hash::ObjectId;
use helpers::new_client;
use std::str::FromStr;
pub use types::*;
use vastrum_frontend_lib::get_pub_key;
use vastrum_git_lib::universal::{
    differ::diff_commits,
    directory_explorer::{ExplorerEntry, get_file_data, get_files_for_tree},
    merger::{MergeMode, MergeResult, merge_branches, merge_repos},
    utils::{GitContext, sha1_to_oid},
};
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::crypto::sha256::sha256_hash;
use wasm_bindgen::prelude::*;
