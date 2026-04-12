#[cfg(test)]
mod tests {
    use vastrum_rpc_client::SentTxBehavior;
    use serial_test::serial;

    use crate::ContractAbiClient;
    use crate::native::upload::push_to_repo;
    use crate::testing::test_helpers::TestContext;
    use crate::universal::merger::{MergeMode, MergeResult, merge_repos};

    async fn create_and_init_repo(repo_name: &str, contract: &ContractAbiClient) {
        contract.create_repository(repo_name, "").await.await_confirmation().await;
        let path = format!("test_repos/{repo_name}");
        push_to_repo(&path, repo_name, contract, None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_push_divergent_rejected() {
        let ctx = TestContext::new().await;

        //push initial repo state to smart contract

        create_and_init_repo("test_repo", &ctx.contract).await;

        //create feature branch 1 (feature_branch_1_test_repo)
        create_and_init_repo("feature-branch-1", &ctx.contract).await;

        //create feature branch 2 (feature_branch_2_test_repo)
        create_and_init_repo("feature-branch-2", &ctx.contract).await;

        //merge feature branch 1, should give fast forward
        let merge_res =
            merge_repos("test_repo", "master", "feature-branch-1", "master", &ctx.contract, MergeMode::Live)
                .await
                .unwrap();
        assert!(matches!(merge_res, MergeResult::FastForward(_)));

        //merge branch 1 again, should give uptodate
        let merge_res =
            merge_repos("test_repo", "master", "feature-branch-1", "master", &ctx.contract, MergeMode::Live)
                .await
                .unwrap();
        assert!(matches!(merge_res, MergeResult::AlreadyUpToDate));

        //merge feature branch 2 which has diverged from base repo
        //,should give ::merged as there is no conflicts

        let merge_res =
            merge_repos("test_repo", "master", "feature-branch-2", "master", &ctx.contract, MergeMode::Live)
                .await
                .unwrap();
        assert!(matches!(merge_res, MergeResult::Merged(_)));

        //create feature branch 3, should conflict with rest of branch and be unresolvable
        create_and_init_repo("feature-branch-3", &ctx.contract).await;
        let merge_res =
            merge_repos("test_repo", "master", "feature-branch-3", "master", &ctx.contract, MergeMode::Live)
                .await
                .unwrap();
        assert!(matches!(merge_res, MergeResult::Conflict(_)));
    }
}
