use crate::types::*;
use vastrum_git_lib::universal::utils::sha1_to_oid;

pub fn convert_issue_reply(reply: &vastrum_git_lib::IssueReply) -> IssueReply {
    IssueReply {
        content: reply.content.clone(),
        timestamp: reply.timestamp,
        from: reply.from.to_string(),
    }
}

pub fn convert_issue(issue: &vastrum_git_lib::Issue) -> Issue {
    Issue {
        id: issue.id,
        title: issue.title.clone(),
        description: issue.description.clone(),
        timestamp: issue.timestamp,
        from: issue.from.to_string(),
        reply_count: issue.reply_count,
    }
}

pub fn convert_discussion_reply(reply: &vastrum_git_lib::DiscussionReply) -> DiscussionReply {
    DiscussionReply {
        content: reply.content.clone(),
        timestamp: reply.timestamp,
        from: reply.from.to_string(),
    }
}

pub fn convert_discussion(discussion: &vastrum_git_lib::Discussion) -> Discussion {
    Discussion {
        id: discussion.id,
        title: discussion.title.clone(),
        description: discussion.description.clone(),
        timestamp: discussion.timestamp,
        from: discussion.from.to_string(),
        reply_count: discussion.reply_count,
    }
}

pub fn convert_pr_reply(reply: &vastrum_git_lib::PullRequestReply) -> PullRequestReply {
    PullRequestReply {
        content: reply.content.clone(),
        timestamp: reply.timestamp,
        from: reply.from.to_string(),
    }
}

pub fn convert_pull_request(pr: &vastrum_git_lib::PullRequest) -> PullRequest {
    PullRequest {
        id: pr.id,
        title: pr.title.clone(),
        description: pr.description.clone(),
        merging_repo: pr.merging_repo.clone(),
        reply_count: pr.reply_count,
        is_open: pr.is_open,
        is_merged: pr.is_merged,
        from: pr.from.to_string(),
        created_at: pr.created_at,
    }
}

pub fn convert_git_repository(repo: &vastrum_git_lib::GitRepository) -> GitRepository {
    let head_commit_hash = match &repo.head_commit_hash {
        Some(h) => sha1_to_oid(h).to_string(),
        None => String::new(),
    };
    GitRepository {
        name: repo.name.clone(),
        description: repo.description.clone(),
        owner: repo.owner.to_string(),
        head_commit_hash,
    }
}
