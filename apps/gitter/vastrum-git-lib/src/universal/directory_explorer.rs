#[derive(Serialize, Debug, Clone, PartialEq, Eq, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ExplorerEntry {
    pub name: String,
    pub is_directory: bool,
    pub oid: String,
}

//for initial preview
pub async fn get_top_level_files(
    repo_name: &str,
    branch: &str,
    contract: &ContractAbiClient,
) -> Result<Vec<ExplorerEntry>> {
    let state = contract.state().await;
    let ctx = GitContext::new(state.git_object_store);
    let head = get_head_commit(&state.repo_store, repo_name, branch).await?;
    let head_commit = ctx.read_commit(head).await?;
    return get_files_for_tree(head_commit.tree, &ctx).await;
}

//for subfolder exploration
pub async fn get_files_for_tree(tree_id: ObjectId, ctx: &GitContext) -> Result<Vec<ExplorerEntry>> {
    let head_tree = ctx.read_tree(tree_id).await?;
    let mut top_level_files = vec![];
    for entry in head_tree.entries {
        let is_directory = entry.mode.is_tree();
        let entry = ExplorerEntry {
            name: entry.filename.to_string(),
            is_directory,
            oid: entry.oid.to_string(),
        };
        top_level_files.push(entry);
    }
    top_level_files.sort_by(|a, b| {
        let directories_first = b.is_directory.cmp(&a.is_directory);
        let alphabetical = a.name.cmp(&b.name);
        return directories_first.then(alphabetical);
    });
    return Ok(top_level_files);
}

pub async fn get_file_data(blob_id: ObjectId, ctx: &GitContext) -> Result<Vec<u8>> {
    let blob = ctx.blob_read(blob_id).await?;
    return Ok(blob.data);
}

use crate::{
    ContractAbiClient,
    error::Result,
    universal::utils::{GitContext, get_head_commit},
};
use gix_hash::ObjectId;
use serde::Serialize;
use tsify::Tsify;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::upload::push_to_repo;
    use crate::testing::test_helpers::{TestContext, TestRepoBuilder};
    use serial_test::serial;
    use std::str::FromStr;
    use vastrum_rpc_client::SentTxBehavior;

    #[tokio::test]
    #[serial]
    async fn test_directory_explorer() {
        let ctx = TestContext::new().await;
        let contract = &ctx.contract;
        let git_ctx = GitContext::from_contract(contract).await;
        let repo_name = "explorer_test";

        let test_repo = TestRepoBuilder::new()
            .file("README.md", b"# README")
            .file("binary.bin", &[0x00, 0x01, 0xFF, 0xFE])
            .dir("empty")
            .file("subdir/nested.txt", b"nested content")
            .file("subdir/deep/deep.txt", b"deep content")
            .build();

        contract.create_repository(repo_name, "test").await.await_confirmation().await;

        push_to_repo(test_repo.path_str(), repo_name, contract, None).await.unwrap();

        // Test get_top_level_files
        let top_files = get_top_level_files(repo_name, "main", contract).await.unwrap();
        assert_eq!(top_files.len(), 4);

        let readme = top_files.iter().find(|e| e.name == "README.md").unwrap();
        assert!(!readme.is_directory);

        let subdir = top_files.iter().find(|e| e.name == "subdir").unwrap();
        assert!(subdir.is_directory);

        // Test get_files_for_tree (subfolder)
        let subdir_files =
            get_files_for_tree(ObjectId::from_str(&subdir.oid).unwrap(), &git_ctx).await.unwrap();
        assert_eq!(subdir_files.len(), 2);

        let nested = subdir_files.iter().find(|e| e.name == "nested.txt").unwrap();
        assert!(!nested.is_directory);

        let deep = subdir_files.iter().find(|e| e.name == "deep").unwrap();
        assert!(deep.is_directory);

        // Test get_file_data
        let readme_data =
            get_file_data(ObjectId::from_str(&readme.oid).unwrap(), &git_ctx).await.unwrap();
        assert_eq!(readme_data, b"# README");

        let nested_data =
            get_file_data(ObjectId::from_str(&nested.oid).unwrap(), &git_ctx).await.unwrap();
        assert_eq!(nested_data, b"nested content");

        // Test deeply nested
        let deep_files =
            get_files_for_tree(ObjectId::from_str(&deep.oid).unwrap(), &git_ctx).await.unwrap();
        assert_eq!(deep_files.len(), 1);
        assert_eq!(deep_files[0].name, "deep.txt");

        let deep_data =
            get_file_data(ObjectId::from_str(&deep_files[0].oid).unwrap(), &git_ctx).await.unwrap();
        assert_eq!(deep_data, b"deep content");

        // Test empty directory
        let empty_dir = top_files.iter().find(|e| e.name == "empty").unwrap();
        assert!(empty_dir.is_directory);
        let empty_files = get_files_for_tree(ObjectId::from_str(&empty_dir.oid).unwrap(), &git_ctx)
            .await
            .unwrap();
        assert_eq!(empty_files.len(), 0);

        // Test binary content preservation
        let binary = top_files.iter().find(|e| e.name == "binary.bin").unwrap();
        assert!(!binary.is_directory);
        let binary_data =
            get_file_data(ObjectId::from_str(&binary.oid).unwrap(), &git_ctx).await.unwrap();
        assert_eq!(binary_data, vec![0x00, 0x01, 0xFF, 0xFE]);
    }
}
