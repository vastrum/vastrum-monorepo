pub async fn diff_repos(
    repo1: &str,
    branch1: &str,
    repo2: &str,
    branch2: &str,
    contract: &ContractAbiClient,
) -> Result<RepoDiff> {
    let state = contract.state().await;
    let ctx = GitContext::new(state.git_object_store);
    let commit1 = get_head_commit(&state.repo_store, repo1, branch1).await?;
    let commit2 = get_head_commit(&state.repo_store, repo2, branch2).await?;
    return diff_commits(commit1, commit2, &ctx).await;
}

pub async fn diff_commits(
    commit1: ObjectId,
    commit2: ObjectId,
    ctx: &GitContext,
) -> Result<RepoDiff> {
    let tree1 = ctx.read_commit(commit1).await?.tree;
    let tree2 = ctx.read_commit(commit2).await?.tree;

    let mut diffs = Vec::new();
    diff_trees(tree1, tree2, "", &mut diffs, ctx).await?;

    let mut total_additions = 0;
    let mut total_deletions = 0;
    for diff in &diffs {
        total_additions += diff.additions;
        total_deletions += diff.deletions;
    }

    let repo_diff = RepoDiff { files: diffs, total_additions, total_deletions };
    return Ok(repo_diff);
}

pub fn generate_diff(old_content: &str, new_content: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let longest_common_subsequence = longest_common_subsequence(&old_lines, &new_lines);
    let mut diff_lines = Vec::new();
    let mut old_idx = 0;
    let mut new_idx = 0;

    for (old_match, new_match) in longest_common_subsequence {
        for i in old_idx..old_match {
            diff_lines.push(DiffLine::remove(old_lines[i]));
        }
        for i in new_idx..new_match {
            diff_lines.push(DiffLine::add(new_lines[i]));
        }
        diff_lines.push(DiffLine::context(old_lines[old_match]));
        old_idx = old_match + 1;
        new_idx = new_match + 1;
    }

    for i in old_idx..old_lines.len() {
        diff_lines.push(DiffLine::remove(old_lines[i]));
    }
    for i in new_idx..new_lines.len() {
        diff_lines.push(DiffLine::add(new_lines[i]));
    }

    return diff_lines;
}

fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() { name.to_string() } else { format!("{prefix}/{name}") }
}

async fn collect_removed(
    oid: ObjectId,
    mode: EntryMode,
    path: &str,
    diffs: &mut Vec<FileDiff>,
    ctx: &GitContext,
) -> Result<()> {
    if mode.is_tree() {
        for (name, (oid, mode)) in ctx.read_tree_entries(Some(oid)).await? {
            let child = join_path(path, &name);
            Box::pin(collect_removed(oid, mode, &child, diffs, ctx)).await?;
        }
    } else {
        match read_blob_text(oid, ctx).await? {
            None => diffs.push(binary_diff(path.to_string(), FileStatus::Deleted)),
            Some(c) => diffs.push(create_file_diff(path, FileStatus::Deleted, &c, "")),
        }
    }
    return Ok(());
}

async fn collect_added(
    oid: ObjectId,
    mode: EntryMode,
    path: &str,
    diffs: &mut Vec<FileDiff>,
    ctx: &GitContext,
) -> Result<()> {
    if mode.is_tree() {
        for (name, (oid, mode)) in ctx.read_tree_entries(Some(oid)).await? {
            let child = join_path(path, &name);
            Box::pin(collect_added(oid, mode, &child, diffs, ctx)).await?;
        }
    } else {
        match read_blob_text(oid, ctx).await? {
            None => diffs.push(binary_diff(path.to_string(), FileStatus::Added)),
            Some(c) => diffs.push(create_file_diff(path, FileStatus::Added, "", &c)),
        }
    }
    return Ok(());
}

async fn diff_trees(
    old_tree: ObjectId,
    new_tree: ObjectId,
    prefix: &str,
    diffs: &mut Vec<FileDiff>,
    ctx: &GitContext,
) -> Result<()> {
    if old_tree == new_tree {
        return Ok(());
    }

    let old_entries = ctx.read_tree_entries(Some(old_tree)).await?;
    let new_entries = ctx.read_tree_entries(Some(new_tree)).await?;
    let all_names: HashSet<_> = old_entries.keys().chain(new_entries.keys()).cloned().collect();

    for name in all_names {
        let old = old_entries.get(&name).copied();
        let new = new_entries.get(&name).copied();
        if old.map(|e| e.0) == new.map(|e| e.0) {
            continue;
        }

        let path = join_path(prefix, &name);

        match (old, new) {
            //new commit added file
            (None, Some((oid, mode))) => {
                collect_added(oid, mode, &path, diffs, ctx).await?;
            }
            //new commit removed file
            (Some((oid, mode)), None) => {
                collect_removed(oid, mode, &path, diffs, ctx).await?;
            }
            //new commit changed old preexisting file
            (Some((old_oid, old_mode)), Some((new_oid, new_mode))) => {
                let both_oids_are_trees = old_mode.is_tree() && new_mode.is_tree();
                let both_oids_are_blobs = !old_mode.is_tree() && !new_mode.is_tree();
                let changed_between_tree_and_blob = !both_oids_are_trees && !both_oids_are_blobs;

                if both_oids_are_trees {
                    Box::pin(diff_trees(old_oid, new_oid, &path, diffs, ctx)).await?;
                } else if both_oids_are_blobs {
                    diff_blobs(old_oid, new_oid, &path, diffs, ctx).await?;
                } else if changed_between_tree_and_blob {
                    collect_removed(old_oid, old_mode, &path, diffs, ctx).await?;
                    collect_added(new_oid, new_mode, &path, diffs, ctx).await?;
                }
            }
            (None, None) => unreachable!(),
        }
    }
    return Ok(());
}

async fn diff_blobs(
    old_oid: ObjectId,
    new_oid: ObjectId,
    path: &str,
    diffs: &mut Vec<FileDiff>,
    ctx: &GitContext,
) -> Result<()> {
    let old_text = read_blob_text(old_oid, ctx).await?;
    let new_text = read_blob_text(new_oid, ctx).await?;
    let blob_diff = match (old_text, new_text) {
        (Some(old), Some(new)) => create_file_diff(path, FileStatus::Modified, &old, &new),
        _ => binary_diff(path.to_string(), FileStatus::ModifiedBinary),
    };
    diffs.push(blob_diff);
    return Ok(());
}

async fn read_blob_text(oid: ObjectId, ctx: &GitContext) -> Result<Option<String>> {
    let blob = ctx.blob_read(oid).await?;
    let text = if is_binary(&blob.data) {
        None
    } else {
        Some(String::from_utf8_lossy(&blob.data).into_owned())
    };
    return Ok(text);
}

fn longest_common_subsequence<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<(usize, usize)> {
    let m = a.len();
    let n = b.len();

    if m == 0 || n == 0 {
        return vec![];
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut matched_pairs = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            matched_pairs.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    matched_pairs.reverse();
    return matched_pairs;
}

fn is_binary(data: &[u8]) -> bool {
    let has_null_bytes = data.iter().take(8000).any(|&b| b == 0);
    let is_binary = has_null_bytes;
    return is_binary;
}

fn binary_diff(path: String, status: FileStatus) -> FileDiff {
    let file_diff = FileDiff { path, status, additions: 0, deletions: 0, diff: vec![] };
    return file_diff;
}

fn create_file_diff(path: impl Into<String>, status: FileStatus, old: &str, new: &str) -> FileDiff {
    let diff_lines = generate_diff(old, new);
    let additions = diff_lines.iter().filter(|l| l.line_type == DiffLineType::Add).count() as u32;
    let deletions =
        diff_lines.iter().filter(|l| l.line_type == DiffLineType::Remove).count() as u32;
    let file_diff = FileDiff { path: path.into(), status, additions, deletions, diff: diff_lines };
    return file_diff;
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Tsify)]
#[tsify(into_wasm_abi)]
pub enum DiffLineType {
    Add,
    Remove,
    Context,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Tsify)]
#[tsify(into_wasm_abi)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
}

impl DiffLine {
    pub fn add(content: impl Into<String>) -> Self {
        Self { line_type: DiffLineType::Add, content: content.into() }
    }

    pub fn remove(content: impl Into<String>) -> Self {
        Self { line_type: DiffLineType::Remove, content: content.into() }
    }

    pub fn context(content: impl Into<String>) -> Self {
        Self { line_type: DiffLineType::Context, content: content.into() }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    ModifiedBinary,
}

#[derive(Serialize, Debug, Clone, Tsify)]
#[tsify(into_wasm_abi)]
pub struct FileDiff {
    pub path: String,
    pub status: FileStatus,
    pub additions: u32,
    pub deletions: u32,
    pub diff: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct RepoDiff {
    pub files: Vec<FileDiff>,
    pub total_additions: u32,
    pub total_deletions: u32,
}

use crate::ContractAbiClient;
use crate::error::Result;
use crate::universal::utils::{GitContext, get_head_commit};
use gix_hash::ObjectId;
use gix_object::tree::EntryMode;
use serde::Serialize;
use std::collections::HashSet;
use tsify::Tsify;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::upload::push_to_repo;
    use crate::testing::test_helpers::{TestContext, TestRepoBuilder};

    use serial_test::serial;
    use vastrum_rpc_client::SentTxBehavior;

    #[tokio::test]
    #[serial]
    async fn test_diff_repos_added_file() {
        let ctx = TestContext::new().await;

        let repo1_name = "test_diff_added_repo1";
        let repo2_name = "test_diff_added_repo2";

        let repo1 = TestRepoBuilder::new().file("file1.txt", b"hello world").build();
        ctx.contract.create_repository(repo1_name, "test").await.await_confirmation().await;
        push_to_repo(repo1.path_str(), repo1_name, &ctx.contract, None).await.unwrap();

        let repo2 = TestRepoBuilder::new()
            .file("file1.txt", b"hello world")
            .file("file2.txt", b"new file content\nline 2\nline 3")
            .build();
        ctx.contract.create_repository(repo2_name, "test").await.await_confirmation().await;
        push_to_repo(repo2.path_str(), repo2_name, &ctx.contract, None).await.unwrap();

        let diff = diff_repos(repo1_name, "master", repo2_name, "master", &ctx.contract).await.unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, "file2.txt");
        assert_eq!(diff.files[0].status, FileStatus::Added);
        assert_eq!(diff.files[0].additions, 3);
        assert_eq!(diff.files[0].deletions, 0);
        assert_eq!(diff.total_additions, 3);
        assert_eq!(diff.total_deletions, 0);
        assert!(diff.files[0].diff.iter().any(|l| l.line_type == DiffLineType::Add));
    }

    #[tokio::test]
    #[serial]
    async fn test_diff_repos_modified_file() {
        let ctx = TestContext::new().await;

        let repo1_name = "test_diff_modified_repo1";
        let repo2_name = "test_diff_modified_repo2";

        let repo1 = TestRepoBuilder::new().file("file.txt", b"line 1\nline 2\nline 3\n").build();
        ctx.contract.create_repository(repo1_name, "test").await.await_confirmation().await;
        push_to_repo(repo1.path_str(), repo1_name, &ctx.contract, None).await.unwrap();

        let repo2 = TestRepoBuilder::new()
            .file("file.txt", b"line 1\nmodified line 2\nline 3\nline 4\n")
            .build();
        ctx.contract.create_repository(repo2_name, "test").await.await_confirmation().await;
        push_to_repo(repo2.path_str(), repo2_name, &ctx.contract, None).await.unwrap();

        let diff = diff_repos(repo1_name, "master", repo2_name, "master", &ctx.contract).await.unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, "file.txt");
        assert_eq!(diff.files[0].status, FileStatus::Modified);
        assert!(diff.files[0].additions > 0);
        assert!(diff.files[0].deletions > 0);
        assert!(diff.files[0].diff.iter().any(|l| l.line_type == DiffLineType::Add));
        assert!(diff.files[0].diff.iter().any(|l| l.line_type == DiffLineType::Remove));
    }

    #[tokio::test]
    #[serial]
    async fn test_diff_repos_deleted_file() {
        let ctx = TestContext::new().await;

        let repo1_name = "test_diff_deleted_repo1";
        let repo2_name = "test_diff_deleted_repo2";

        let repo1 = TestRepoBuilder::new()
            .file("file1.txt", b"file 1 content")
            .file("file2.txt", b"file 2 content\nto be deleted")
            .build();
        ctx.contract.create_repository(repo1_name, "test").await.await_confirmation().await;
        push_to_repo(repo1.path_str(), repo1_name, &ctx.contract, None).await.unwrap();

        let repo2 = TestRepoBuilder::new().file("file1.txt", b"file 1 content").build();
        ctx.contract.create_repository(repo2_name, "test").await.await_confirmation().await;
        push_to_repo(repo2.path_str(), repo2_name, &ctx.contract, None).await.unwrap();

        let diff = diff_repos(repo1_name, "master", repo2_name, "master", &ctx.contract).await.unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, "file2.txt");
        assert_eq!(diff.files[0].status, FileStatus::Deleted);
        assert_eq!(diff.files[0].additions, 0);
        assert_eq!(diff.files[0].deletions, 2);
        assert!(diff.files[0].diff.iter().any(|l| l.line_type == DiffLineType::Remove));
        assert!(!diff.files[0].diff.iter().any(|l| l.line_type == DiffLineType::Add));
    }

    #[tokio::test]
    #[serial]
    async fn test_diff_commits_multiple_changes() {
        let ctx = TestContext::new().await;

        let repo_name = "test_diff_multiple";

        let local = TestRepoBuilder::new()
            .file("a.txt", b"original a\n")
            .file("b.txt", b"original b\n")
            .file("c.txt", b"original c\nwill be deleted\n")
            .build();
        let commit1 = local.head_id();
        let commit2 = local.add_commit(&[
            ("a.txt", b"modified a\nnew line\n"),
            ("b.txt", b"original b\n"),
            ("d.txt", b"new file d\n"),
        ]);

        ctx.contract.create_repository(repo_name, "test").await.await_confirmation().await;
        push_to_repo(local.path_str(), repo_name, &ctx.contract, None).await.unwrap();

        let git_ctx = GitContext::from_contract(&ctx.contract).await;
        let diff = diff_commits(commit1, commit2, &git_ctx).await.unwrap();

        assert_eq!(diff.files.len(), 3);

        let a_diff = diff.files.iter().find(|f| f.path == "a.txt").unwrap();
        let c_diff = diff.files.iter().find(|f| f.path == "c.txt").unwrap();
        let d_diff = diff.files.iter().find(|f| f.path == "d.txt").unwrap();

        assert_eq!(a_diff.status, FileStatus::Modified);
        assert_eq!(c_diff.status, FileStatus::Deleted);
        assert_eq!(d_diff.status, FileStatus::Added);

        assert!(a_diff.diff.iter().any(|l| l.line_type == DiffLineType::Add));
        assert!(a_diff.diff.iter().any(|l| l.line_type == DiffLineType::Remove));
        assert!(c_diff.diff.iter().any(|l| l.line_type == DiffLineType::Remove));
        assert!(d_diff.diff.iter().any(|l| l.line_type == DiffLineType::Add));
    }

    #[tokio::test]
    #[serial]
    async fn test_diff_nested_directories() {
        let ctx = TestContext::new().await;

        let repo_name = "test_diff_nested";

        let local = TestRepoBuilder::new().file("src/main.rs", b"fn main() {}\n").build();
        let commit1 = local.head_id();
        let commit2 = local.add_commit(&[
            ("src/main.rs", b"fn main() {\n    println!(\"hello\");\n}\n"),
            ("src/lib.rs", b"pub fn lib() {}\n"),
        ]);

        ctx.contract.create_repository(repo_name, "test").await.await_confirmation().await;
        push_to_repo(local.path_str(), repo_name, &ctx.contract, None).await.unwrap();

        let git_ctx = GitContext::from_contract(&ctx.contract).await;
        let diff = diff_commits(commit1, commit2, &git_ctx).await.unwrap();

        assert_eq!(diff.files.len(), 2);

        let main_diff = diff.files.iter().find(|f| f.path == "src/main.rs").unwrap();
        let lib_diff = diff.files.iter().find(|f| f.path == "src/lib.rs").unwrap();

        assert_eq!(main_diff.status, FileStatus::Modified);
        assert_eq!(lib_diff.status, FileStatus::Added);

        assert!(
            main_diff
                .diff
                .iter()
                .any(|l| l.line_type == DiffLineType::Add || l.line_type == DiffLineType::Remove)
        );
        assert!(lib_diff.diff.iter().any(|l| l.line_type == DiffLineType::Add));
    }

    #[test]
    fn test_create_file_diff_basic() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nmodified line 2\nline 3\nline 4\n";

        let file_diff = create_file_diff("test.txt", FileStatus::Modified, old, new);

        assert!(file_diff.additions > 0);
        assert!(file_diff.deletions > 0);
        assert!(file_diff.diff.iter().any(|l| l.line_type == DiffLineType::Add));
        assert!(file_diff.diff.iter().any(|l| l.line_type == DiffLineType::Remove));
    }

    #[test]
    fn test_create_file_diff_empty_to_content() {
        let file_diff = create_file_diff("new.txt", FileStatus::Added, "", "new\nfile\n");

        assert_eq!(file_diff.additions, 2);
        assert_eq!(file_diff.deletions, 0);
        assert!(
            file_diff.diff.iter().any(|l| l.line_type == DiffLineType::Add && l.content == "new")
        );
    }

    #[test]
    fn test_create_file_diff_content_to_empty() {
        let file_diff = create_file_diff("deleted.txt", FileStatus::Deleted, "old\ncontent\n", "");

        assert_eq!(file_diff.additions, 0);
        assert_eq!(file_diff.deletions, 2);
        assert!(
            file_diff
                .diff
                .iter()
                .any(|l| l.line_type == DiffLineType::Remove && l.content == "old")
        );
    }

    #[test]
    fn test_is_binary() {
        assert!(!is_binary(b"hello world"));
        assert!(!is_binary(b"line 1\nline 2\nline 3\n"));
        assert!(!is_binary(b""));

        assert!(is_binary(b"hello\0world"));
        assert!(is_binary(b"\0"));
        assert!(is_binary(&[0u8; 100]));

        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert!(is_binary(&png_header));
    }

    #[tokio::test]
    #[serial]
    async fn test_diff_binary_file() {
        let ctx = TestContext::new().await;

        let repo1_name = "test_diff_binary_repo1";
        let repo2_name = "test_diff_binary_repo2";

        let repo1 = TestRepoBuilder::new().file("file.txt", b"hello world").build();
        ctx.contract.create_repository(repo1_name, "test").await.await_confirmation().await;
        push_to_repo(repo1.path_str(), repo1_name, &ctx.contract, None).await.unwrap();

        let repo2 = TestRepoBuilder::new()
            .file("file.txt", b"hello world")
            .file("image.png", b"PNG\x89\x00\x00binary\x00data")
            .build();
        ctx.contract.create_repository(repo2_name, "test").await.await_confirmation().await;
        push_to_repo(repo2.path_str(), repo2_name, &ctx.contract, None).await.unwrap();

        let diff = diff_repos(repo1_name, "master", repo2_name, "master", &ctx.contract).await.unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, "image.png");
        assert_eq!(diff.files[0].status, FileStatus::Added);
        assert_eq!(diff.files[0].additions, 0);
        assert_eq!(diff.files[0].deletions, 0);
        assert!(diff.files[0].diff.is_empty());
    }
}
