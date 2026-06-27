use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub html_url: String,
    pub head: Branch,
    pub base: Branch,
    pub draft: bool,
    pub mergeable: Option<bool>,
    pub mergeable_state: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Branch {
    pub label: String,
    pub r#ref: String,
    pub sha: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    pub private: bool,
    pub html_url: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub language: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub html_url: String,
    pub workflow_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusCheck {
    pub state: String,
    pub description: Option<String>,
    pub context: String,
    pub target_url: Option<String>,
}


impl PullRequest {
    pub fn status_emoji(&self) -> &'static str {
        match self.state.as_str() {
            "open" => "🟢",
            "closed" => "🔴",
            "merged" => "🟣",
            _ => "⚪",
        }
    }

    pub fn mergeable_emoji(&self) -> &'static str {
        match self.mergeable_state.as_deref() {
            Some("clean") => "✅",
            Some("dirty") => "⚠️",
            Some("unstable") => "❌",
            Some("blocked") => "🚫",
            _ => "❓",
        }
    }

    pub fn is_draft(&self) -> bool {
        self.draft
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_pr(state: &str, draft: bool, mergeable_state: Option<&str>) -> PullRequest {
        PullRequest {
            number: 1,
            title: "test".to_string(),
            state: state.to_string(),
            user: User { login: "user".to_string(), id: 1, avatar_url: String::new() },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            html_url: String::new(),
            head: Branch { label: "head".to_string(), r#ref: "head".to_string(), sha: String::new() },
            base: Branch { label: "main".to_string(), r#ref: "main".to_string(), sha: String::new() },
            draft,
            mergeable: None,
            mergeable_state: mergeable_state.map(str::to_string),
        }
    }

    #[test]
    fn test_status_emoji_open() {
        assert_eq!(make_pr("open", false, None).status_emoji(), "🟢");
    }

    #[test]
    fn test_status_emoji_closed() {
        assert_eq!(make_pr("closed", false, None).status_emoji(), "🔴");
    }

    #[test]
    fn test_status_emoji_merged() {
        assert_eq!(make_pr("merged", false, None).status_emoji(), "🟣");
    }

    #[test]
    fn test_status_emoji_unknown() {
        assert_eq!(make_pr("unknown", false, None).status_emoji(), "⚪");
    }

    #[test]
    fn test_mergeable_emoji_clean() {
        assert_eq!(make_pr("open", false, Some("clean")).mergeable_emoji(), "✅");
    }

    #[test]
    fn test_mergeable_emoji_dirty() {
        assert_eq!(make_pr("open", false, Some("dirty")).mergeable_emoji(), "⚠️");
    }

    #[test]
    fn test_mergeable_emoji_blocked() {
        assert_eq!(make_pr("open", false, Some("blocked")).mergeable_emoji(), "🚫");
    }

    #[test]
    fn test_mergeable_emoji_none() {
        assert_eq!(make_pr("open", false, None).mergeable_emoji(), "❓");
    }

    #[test]
    fn test_is_draft_true() {
        assert!(make_pr("open", true, None).is_draft());
    }

    #[test]
    fn test_is_draft_false() {
        assert!(!make_pr("open", false, None).is_draft());
    }
}
