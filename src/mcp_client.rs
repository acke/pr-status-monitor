use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<MCPError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
}

pub struct MCPClient {
    client: Client,
    base_url: String,
    token: String,
}

impl MCPClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }

    pub async fn call_github_api(&self, endpoint: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "prtracker-cli")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub API request failed: {}",
                response.status()
            ));
        }

        let json: serde_json::Value = response.json().await?;
        Ok(json)
    }

    pub async fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        per_page: usize,
    ) -> Result<Vec<serde_json::Value>> {
        let endpoint = format!(
            "repos/{}/{}/pulls?state={}&per_page={}&sort=updated&direction=desc",
            owner, repo, state, per_page
        );
        
        let response = self.call_github_api(&endpoint).await?;
        
        match response.as_array() {
            Some(prs) => Ok(prs.clone()),
            None => Err(anyhow!("Invalid response format from GitHub API")),
        }
    }

    pub async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<serde_json::Value> {
        let endpoint = format!("repos/{}/{}/pulls/{}", owner, repo, pr_number);
        self.call_github_api(&endpoint).await
    }

    pub async fn get_pr_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<serde_json::Value> {
        let endpoint = format!("repos/{}/{}/pulls/{}/reviews", owner, repo, pr_number);
        self.call_github_api(&endpoint).await
    }

    pub async fn list_user_repositories(&self, limit: usize) -> Result<Vec<serde_json::Value>> {
        let endpoint = format!("user/repos?sort=updated&per_page={}&type=all", limit);
        let response = self.call_github_api(&endpoint).await?;
        
        match response.as_array() {
            Some(repos) => Ok(repos.clone()),
            None => Err(anyhow!("Invalid response format from GitHub API")),
        }
    }

    #[allow(dead_code)]
    pub async fn search_user_pull_requests(
        &self,
        username: &str,
        state: &str,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>> {
        let query = format!("is:pr author:{} state:{}", username, state);
        let endpoint = format!(
            "search/issues?q={}&sort=updated&per_page={}",
            urlencoding::encode(&query),
            limit
        );
        
        let response = self.call_github_api(&endpoint).await?;
        
        match response.get("items").and_then(|items| items.as_array()) {
            Some(prs) => Ok(prs.clone()),
            None => Err(anyhow!("Invalid response format from GitHub API")),
        }
    }

    pub async fn get_pr_status_checks(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<serde_json::Value> {
        // Use the statuses endpoint to get individual status checks
        let endpoint = format!("repos/{}/{}/commits/{}/statuses", owner, repo, sha);
        self.call_github_api(&endpoint).await
    }

    pub async fn get_combined_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<serde_json::Value> {
        // Get the combined status that GitHub shows in the UI
        let endpoint = format!("repos/{}/{}/commits/{}/status", owner, repo, sha);
        self.call_github_api(&endpoint).await
    }

    pub async fn get_pr_check_runs(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<serde_json::Value> {
        let endpoint = format!("repos/{}/{}/commits/{}/check-runs", owner, repo, sha);
        self.call_github_api(&endpoint).await
    }

    pub async fn get_pr_workflow_runs(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<serde_json::Value> {
        // Get workflow runs for the PR - get more recent runs and increase limit
        let endpoint = format!(
            "repos/{}/{}/actions/runs?event=pull_request&per_page=50",
            owner, repo
        );
        
        let response = self.call_github_api(&endpoint).await?;
        
        // Filter runs for this specific PR, prioritizing recent runs
        if let Some(runs) = response.get("workflow_runs").and_then(|r| r.as_array()) {
            let mut filtered_runs: Vec<serde_json::Value> = runs
                .iter()
                .filter(|run| {
                    if let Some(pull_requests) = run.get("pull_requests").and_then(|pr| pr.as_array()) {
                        pull_requests.iter().any(|pr| {
                            pr.get("number").and_then(|n| n.as_u64()) == Some(pr_number)
                        })
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
            
            // Sort by created_at descending to get most recent runs first
            filtered_runs.sort_by(|a, b| {
                let a_created = a.get("created_at").and_then(|c| c.as_str()).unwrap_or("");
                let b_created = b.get("created_at").and_then(|c| c.as_str()).unwrap_or("");
                b_created.cmp(a_created) // Reverse order for descending
            });
            
            // Take only the most recent 20 runs to avoid stale data
            filtered_runs.truncate(20);
            
            Ok(serde_json::json!({
                "total_count": filtered_runs.len(),
                "workflow_runs": filtered_runs
            }))
        } else {
            Ok(serde_json::json!({
                "total_count": 0,
                "workflow_runs": []
            }))
        }
    }
    
    pub async fn get_workflow_runs_for_commit(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<serde_json::Value> {
        // Get workflow runs for a specific commit SHA - this should be more accurate
        let endpoint = format!(
            "repos/{}/{}/actions/runs?head_sha={}&per_page=20",
            owner, repo, sha
        );
        
        self.call_github_api(&endpoint).await
    }
}
