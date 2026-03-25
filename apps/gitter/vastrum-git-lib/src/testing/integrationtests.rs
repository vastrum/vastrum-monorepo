#[cfg(test)]
mod tests {
    use crate::testing::test_helpers::TestContext;
    use vastrum_rpc_client::SentTxBehavior;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_issues() {
        let ctx = TestContext::new().await;

        let repo_name = "social_tests_repo";
        ctx.contract.create_repository(repo_name, "").await.await_confirmation().await;
        ctx.contract.create_issue("title", "content", repo_name).await.await_confirmation().await;

        ctx.contract.reply_to_issue("reply1", repo_name, 0).await.await_confirmation().await;

        ctx.contract.reply_to_issue("reply2", repo_name, 0).await.await_confirmation().await;

        let repo = ctx.contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();
        let issue_1 = repo.issues.get(0).await.unwrap();
        assert_eq!(issue_1.title, "title");
        assert_eq!(issue_1.description, "content");
        assert_eq!(issue_1.reply_count, 2);
        assert_ne!(issue_1.timestamp, 0);

        assert_eq!(issue_1.replies.get(0).await.unwrap().content, "reply1");
        assert_eq!(issue_1.replies.get(0).await.unwrap().from, ctx.account_key.public_key());
        assert_ne!(issue_1.replies.get(0).await.unwrap().timestamp, 0);

        assert_eq!(issue_1.replies.get(1).await.unwrap().content, "reply2");
        assert_eq!(issue_1.replies.get(1).await.unwrap().from, ctx.account_key.public_key());
        assert_ne!(issue_1.replies.get(1).await.unwrap().timestamp, 0);

        ctx.contract.create_issue("issue2", "issue2", repo_name).await.await_confirmation().await;

        let all_issues =
            ctx.contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap().issues;
        assert_eq!(all_issues.length().await, 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_pull_requests() {
        let ctx = TestContext::new().await;

        let repo_name = "pull_request_tests_repo";
        let merging_repo = "pull_request_test_merge_repo";

        ctx.contract.create_repository(repo_name, "").await.await_confirmation().await;
        ctx.contract.create_repository(merging_repo, "").await.await_confirmation().await;

        ctx.contract
            .create_pull_request(repo_name, merging_repo, "title", "description")
            .await
            .await_confirmation()
            .await;

        ctx.contract.reply_to_pull_request("reply1", repo_name, 0).await.await_confirmation().await;

        ctx.contract.reply_to_pull_request("reply2", repo_name, 0).await.await_confirmation().await;
        let repo = ctx.contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();
        let pull_request_1 = repo.pull_requests.get(0).await.unwrap();
        assert_eq!(pull_request_1.title, "title");
        assert_eq!(pull_request_1.description, "description");
        assert_eq!(pull_request_1.merging_repo, merging_repo);
        assert_eq!(pull_request_1.is_open, true);

        assert_eq!(pull_request_1.replies.get(0).await.unwrap().content, "reply1");
        assert_eq!(pull_request_1.replies.get(0).await.unwrap().from, ctx.account_key.public_key());
        assert_ne!(pull_request_1.replies.get(0).await.unwrap().timestamp, 0);

        assert_eq!(pull_request_1.replies.get(1).await.unwrap().content, "reply2");
        assert_eq!(pull_request_1.replies.get(1).await.unwrap().from, ctx.account_key.public_key());
        assert_ne!(pull_request_1.replies.get(1).await.unwrap().timestamp, 0);

        ctx.contract.close_pull_request(repo_name, 0).await.await_confirmation().await;

        let repo = ctx.contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();
        let pull_request_1 = repo.pull_requests.get(0).await.unwrap();
        assert_eq!(pull_request_1.is_open, false);
    }

    #[tokio::test]
    #[serial]
    async fn test_forks() {
        let ctx = TestContext::new().await;

        let repo_name = "fork_tests_repo";
        let fork_name = "forked_repo";
        let second_fork_name = "second_forked_repo";
        ctx.contract.create_repository(repo_name, "").await.await_confirmation().await;

        ctx.contract.fork_repository(fork_name, repo_name).await.await_confirmation().await;

        ctx.contract.fork_repository(second_fork_name, repo_name).await.await_confirmation().await;

        let forks = ctx
            .contract
            .state()
            .await
            .forks_store
            .get(&ForksKey { repo_name: repo_name.to_string(), from: ctx.account_key.public_key() })
            .await
            .unwrap();
        assert_eq!(forks[0], fork_name);
        assert_eq!(forks[1], second_fork_name);
    }

    use crate::ForksKey;
}
