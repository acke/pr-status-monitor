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
pub struct Review {
    pub id: u64,
    pub user: User,
    pub state: String,
    pub submitted_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub total_count: u64,
    pub items: Vec<serde_json::Value>,
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
