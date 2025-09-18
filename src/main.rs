mod cli;
mod config;
mod github;
mod local;
mod mcp_client;
mod output;

use anyhow::{anyhow, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;
use cli::{Cli, Commands};
use config::Config;
use github::{PullRequest, Repository, WorkflowRun, CheckRun, StatusCheck};
use local::scan_local_repositories;
use mcp_client::MCPClient;
use output::{print_pr_details, print_pr_list, print_repository_list, print_multi_repo_prs, print_my_prs_with_workflow_status, print_pr_status_overview, print_multi_repo_summary};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PRStatusJson {
    repository: String,
    number: u64,
    title: String,
    author: String,
    status: String,
    url: String,
    updated_at: String,
    is_draft: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct StatusSummaryJson {
    total_prs: usize,
    failing_prs: Vec<PRStatusJson>,
    running_prs: Vec<PRStatusJson>,
    passing_prs: Vec<PRStatusJson>,
    review_prs: Vec<PRStatusJson>,
    overall_status: String, // "failing", "running", "passing", "needs_review", "no_prs"
    last_updated: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle config command first
    if let Some(Commands::Config { wizard, show }) = &cli.command {
        return handle_config_command(*wizard, *show).await;
    }
    
    // Load configuration
    let config = Config::load()?;
    
    // Get token from CLI, config, or environment
    let token = cli.token
        .or_else(|| config.get_github_token())
        .ok_or_else(|| {
            anyhow!("GitHub token is required. Set GITHUB_PRIVATE_TOKEN environment variable, use --token, or run 'my-prs config --wizard'")
        })?;
    
    // Initialize MCP client
    let client = MCPClient::new(config.github_base_url.clone(), token);
    
    // Default to my-prs command if no command specified
    let command = cli.command.unwrap_or(Commands::MyPrs { 
        state: None, 
        path: None, 
        username: None 
    });
    
    match command {
        Commands::List { state, limit } => {
            let repo = cli.repo.ok_or_else(|| {
                anyhow!("GitHub repository is required for list command. Set GITHUB_REPO environment variable or use --repo")
            })?;
            let (owner, repo_name) = parse_repo(&repo)?;
            let effective_state = state.unwrap_or(config.default_pr_state.clone());
            let effective_limit = limit.unwrap_or(config.default_pr_limit);
            handle_list_command(&client, &owner, &repo_name, &effective_state, effective_limit).await?;
        }
        Commands::Status { pr_number } => {
            match pr_number {
                Some(pr_num) => {
                    let repo = cli.repo.ok_or_else(|| {
                        anyhow!("GitHub repository is required for status command. Set GITHUB_REPO environment variable or use --repo")
                    })?;
                    let (owner, repo_name) = parse_repo(&repo)?;
                    handle_status_command(&client, &owner, &repo_name, pr_num).await?;
                }
                None => {
                    // Show all open PRs status - use my-prs functionality
                    let effective_state = "open".to_string();
                    let effective_path = config.get_local_repos_path();
                    handle_my_prs_command(&client, &config, &effective_state, &effective_path, None).await?;
                }
            }
        }
        Commands::Repos { limit } => {
            handle_repos_command(&client, limit).await?;
        }
        Commands::Scan { state, limit, max_repos, all, username } => {
            let effective_state = state.unwrap_or(config.default_pr_state.clone());
            let effective_limit = limit.unwrap_or(config.default_pr_limit);
            let effective_max_repos = max_repos.unwrap_or(config.default_repo_limit);
            handle_scan_command(&client, &config, &effective_state, effective_limit, effective_max_repos, all, username.as_deref()).await?;
        }
        Commands::Local { state, limit, path, workflows, all, username } => {
            let effective_state = state.unwrap_or(config.default_pr_state.clone());
            let effective_limit = limit.unwrap_or(config.default_pr_limit);
            let effective_path = path.unwrap_or(config.get_local_repos_path());
            let effective_workflows = workflows.unwrap_or(config.show_workflows_by_default);
            handle_local_command(&client, &config, &effective_state, effective_limit, &effective_path, effective_workflows, all, username.as_deref()).await?;
        }
        Commands::MyPrs { state, path, username } => {
            let effective_state = state.unwrap_or(config.default_pr_state.clone());
            let effective_path = path.unwrap_or(config.get_local_repos_path());
            handle_my_prs_command(&client, &config, &effective_state, &effective_path, username.as_deref()).await?;
        }
        Commands::Monitor { path, interval, username, json } => {
            let effective_path = path.unwrap_or(config.get_local_repos_path());
            handle_monitor_command(&client, &config, &effective_path, interval, username.as_deref(), json).await?;
        }
        Commands::StatusCheck { path, username, json } => {
            let effective_path = path.unwrap_or(config.get_local_repos_path());
            handle_status_check_command(&client, &config, &effective_path, username.as_deref(), json).await?;
        }
        Commands::Config { .. } => {
            // This is handled earlier in main()
            unreachable!()
        }
    }
    
    Ok(())
}

async fn handle_config_command(wizard: bool, show: bool) -> Result<()> {
    if wizard {
        Config::run_wizard()?;
    } else if show {
        let config = Config::load()?;
        println!("📋 Current Configuration:");
        println!("========================");
        println!("GitHub Token: {}", if config.github_token.is_some() { "Set" } else { "Use environment variable" });
        println!("GitHub Username: {}", config.github_username.as_deref().unwrap_or("Auto-detect"));
        println!("GitHub Base URL: {}", config.github_base_url);
        println!("Local Repos Path: {}", config.local_repos_path);
        println!("GitHub Organization: {}", config.github_organization.as_deref().unwrap_or("None"));
        println!("Default PR State: {}", config.default_pr_state);
        println!("Default PR Limit: {}", config.default_pr_limit);
        println!("Default Repo Limit: {}", config.default_repo_limit);
        println!("Show Workflows by Default: {}", config.show_workflows_by_default);
        
        if let Ok(config_path) = Config::config_file_path() {
            println!("\nConfig file location: {}", config_path.display());
        }
    } else {
        println!("Use --wizard to run configuration wizard or --show to display current config");
    }
    Ok(())
}

async fn handle_list_command(
    client: &MCPClient,
    owner: &str,
    repo: &str,
    state: &str,
    limit: usize,
) -> Result<()> {
    println!("Fetching your pull requests for {}/{}...", owner, repo);
    
    let prs_json = client.list_pull_requests(owner, repo, state, limit).await?;
    
    // Get the user's GitHub username for filtering
    let config = Config::load()?;
    let username = config.get_github_username()?;
    
    let prs: Vec<PullRequest> = prs_json
        .into_iter()
        .filter_map(|pr_json| serde_json::from_value(pr_json).ok())
        .filter(|pr: &PullRequest| pr.user.login == username) // Filter to only user's PRs
        .collect();
    
    if prs.is_empty() {
        println!("No pull requests found for @{} in {}/{}", username, owner, repo);
    } else {
        print_pr_list(&prs);
        
        // Add summary for list command
        let org_name = config.github_organization.as_deref().unwrap_or("organization");
        print_pr_status_overview(&prs, &username, org_name);
    }
    
    Ok(())
}

async fn handle_status_command(
    client: &MCPClient,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<()> {
    println!("Fetching details for PR #{}...", pr_number);
    
    let pr_json = client.get_pull_request(owner, repo, pr_number).await?;
    let pr: PullRequest = serde_json::from_value(pr_json)?;
    
    print_pr_details(&pr);
    
    Ok(())
}

async fn handle_repos_command(client: &MCPClient, limit: usize) -> Result<()> {
    println!("Fetching your repositories...");
    
    let repos_json = client.list_user_repositories(limit).await?;
    
    let repos: Vec<Repository> = repos_json
        .into_iter()
        .filter_map(|repo_json| serde_json::from_value(repo_json).ok())
        .collect();
    
    print_repository_list(&repos);
    
    Ok(())
}

async fn handle_scan_command(
    client: &MCPClient,
    config: &Config,
    state: &str,
    limit: usize,
    max_repos: usize,
    show_all: bool,
    username: Option<&str>,
) -> Result<()> {
    // Get username if not showing all PRs
    let filter_username = if show_all {
        None
    } else {
        Some(match username {
            Some(u) => u.to_string(),
            None => config.get_github_username()?,
        })
    };
    
    let org_name = config.github_organization.as_deref().unwrap_or("organization");
    if let Some(ref user) = filter_username {
        println!("Scanning for your PRs (@{}) across {} repositories...", user, org_name);
    } else {
        println!("Scanning for all pull requests across {} repositories...", org_name);
    }
    
    // First get the user's repositories
    let repos_json = client.list_user_repositories(max_repos).await?;
    let repos: Vec<Repository> = repos_json
        .into_iter()
        .filter_map(|repo_json| serde_json::from_value(repo_json).ok())
        .filter(|repo: &Repository| {
            if let Some(org) = &config.github_organization {
                repo.owner.login == *org
            } else {
                true // No organization filter
            }
        })
        .collect();
    
    let mut prs_by_repo = Vec::new();
    
    for repo in repos {
        println!("Checking {}...", repo.full_name);
        
        match client.list_pull_requests(&repo.owner.login, &repo.name, state, limit).await {
            Ok(prs_json) => {
                let mut prs: Vec<PullRequest> = prs_json
                    .into_iter()
                    .filter_map(|pr_json| serde_json::from_value(pr_json).ok())
                    .collect();
                
                // Filter by username if specified
                if let Some(ref user) = filter_username {
                    prs.retain(|pr: &PullRequest| pr.user.login == *user);
                }
                
                if !prs.is_empty() {
                    prs_by_repo.push((repo.full_name, prs));
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch PRs for {}: {}", repo.full_name, e);
            }
        }
    }
    
    print_multi_repo_prs(&prs_by_repo);
    
    // Add summary for scan command
    if let Some(ref user) = filter_username {
        let org_name = config.github_organization.as_deref().unwrap_or("organization");
        print_multi_repo_summary(&prs_by_repo, user, org_name);
    }
    
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_local_command(
    client: &MCPClient,
    config: &Config,
    state: &str,
    limit: usize,
    repos_path: &str,
    show_workflows: bool,
    show_all: bool,
    username: Option<&str>,
) -> Result<()> {
    // Get username if not showing all PRs
    let filter_username = if show_all {
        None
    } else {
        Some(match username {
            Some(u) => u.to_string(),
            None => config.get_github_username()?,
        })
    };
    
    let org_name = config.github_organization.as_deref().unwrap_or("organization");
    if let Some(ref user) = filter_username {
        println!("Scanning for your PRs (@{}) in {} repositories in {}...", user, org_name, repos_path);
    } else {
        println!("Scanning for all PRs in {} repositories in {}...", org_name, repos_path);
    }
    
    // Scan local repositories
    let all_local_repos = scan_local_repositories(repos_path)?;
    let local_repos: Vec<_> = all_local_repos
        .into_iter()
        .filter(|repo| {
            if let Some(org) = &config.github_organization {
                repo.owner == *org
            } else {
                true // No organization filter
            }
        })
        .collect();
    
    if local_repos.is_empty() {
        println!("No git repositories found in {}", repos_path);
        return Ok(());
    }
    
    println!("Found {} local repositories:", local_repos.len());
    for repo in &local_repos {
        println!("  {} -> {}/{}", repo.name, repo.owner, repo.repo);
    }
    println!();
    
    let mut prs_by_repo = Vec::new();
    
    for local_repo in local_repos {
        println!("Checking {}/{}...", local_repo.owner, local_repo.repo);
        
        match client.list_pull_requests(&local_repo.owner, &local_repo.repo, state, limit).await {
            Ok(prs_json) => {
                let mut prs: Vec<PullRequest> = prs_json
                    .into_iter()
                    .filter_map(|pr_json| serde_json::from_value(pr_json).ok())
                    .collect();
                
                // Filter by username if specified
                if let Some(ref user) = filter_username {
                    prs.retain(|pr: &PullRequest| pr.user.login == *user);
                }
                
                // Fetch workflow status if requested
                if show_workflows && !prs.is_empty() {
                    for pr in &mut prs {
                        if let Some(_sha) = pr.head.sha.get(0..7) {
                            // Fetch workflow runs, check runs, and status checks
                            let _workflow_runs = fetch_workflow_runs(client, &local_repo.owner, &local_repo.repo, pr.number).await;
                            let _check_runs = fetch_check_runs(client, &local_repo.owner, &local_repo.repo, &pr.head.sha).await;
                            let _status_checks = fetch_status_checks(client, &local_repo.owner, &local_repo.repo, &pr.head.sha).await;
                            
                            // Store workflow status in PR (we'll need to extend PullRequest struct for this)
                            // For now, we'll handle this in the display logic
                        }
                    }
                }
                
                if !prs.is_empty() {
                    let repo_name = format!("{}/{}", local_repo.owner, local_repo.repo);
                    prs_by_repo.push((repo_name, prs));
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch PRs for {}/{}: {}", 
                         local_repo.owner, local_repo.repo, e);
            }
        }
    }
    
    print_multi_repo_prs(&prs_by_repo);
    
    // Add summary for local command when not showing workflows
    if !show_workflows {
        if let Some(ref user) = filter_username {
            let org_name = config.github_organization.as_deref().unwrap_or("organization");
            print_multi_repo_summary(&prs_by_repo, user, org_name);
        }
    }
    
    Ok(())
}

async fn handle_my_prs_command(
    client: &MCPClient,
    config: &Config,
    state: &str,
    repos_path: &str,
    username: Option<&str>,
) -> Result<()> {
    // Get username from git config if not provided
    let username = match username {
        Some(u) => u.to_string(),
        None => config.get_github_username()?,
    };
    
    let org_name = config.github_organization.as_deref().unwrap_or("organization");
    println!("Scanning for your PRs (@{}) in {} repositories in {}...", username, org_name, repos_path);
    
    // Scan local repositories
    let all_local_repos = scan_local_repositories(repos_path)?;
    let local_repos: Vec<_> = all_local_repos
        .into_iter()
        .filter(|repo| {
            if let Some(org) = &config.github_organization {
                repo.owner == *org
            } else {
                true // No organization filter
            }
        })
        .collect();
    
    if local_repos.is_empty() {
        println!("No git repositories found in {}", repos_path);
        return Ok(());
    }
    
    println!("Found {} local repositories, checking for your PRs...", local_repos.len());
    
    let mut my_prs_by_repo = Vec::new();
    
    for local_repo in local_repos {
        match client.list_pull_requests(&local_repo.owner, &local_repo.repo, state, 50).await {
            Ok(prs_json) => {
                let prs: Vec<PullRequest> = prs_json
                    .into_iter()
                    .filter_map(|pr_json| serde_json::from_value(pr_json).ok())
                    .filter(|pr: &PullRequest| pr.user.login == username) // Filter for user's PRs only
                    .collect();
                
                if !prs.is_empty() {
                    println!("Found {} PR(s) by @{} in {}/{}", 
                            prs.len(), username, local_repo.owner, local_repo.repo);
                    
                    // Fetch workflow status for each PR using the same logic as JSON output
                    let mut prs_with_status = Vec::new();
                    for mut pr in prs {
                        // Get fresh workflow status using the same function as JSON output
                        let workflow_status = get_pr_workflow_status(client, &local_repo.owner, &local_repo.repo, &pr).await;
                        
                        // Also fetch the detailed data for display purposes
                        let workflow_runs = fetch_workflow_runs(client, &local_repo.owner, &local_repo.repo, pr.number).await;
                        let check_runs = fetch_check_runs(client, &local_repo.owner, &local_repo.repo, &pr.head.sha).await;
                        let status_checks = fetch_status_checks(client, &local_repo.owner, &local_repo.repo, &pr.head.sha).await;
                        
                        // Update the PR with fresh timestamp if we got newer data
                        if workflow_status != "No checks" {
                            pr.updated_at = chrono::Utc::now(); // Use current time for fresh data
                        }
                        
                        prs_with_status.push((pr, workflow_runs, check_runs, status_checks));
                    }
                    
                    let repo_name = format!("{}/{}", local_repo.owner, local_repo.repo);
                    my_prs_by_repo.push((repo_name, prs_with_status));
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch PRs for {}/{}: {}", 
                         local_repo.owner, local_repo.repo, e);
            }
        }
    }
    
    let org_name = config.github_organization.as_deref().unwrap_or("organization");
    print_my_prs_with_workflow_status(&my_prs_by_repo, &username, org_name);
    
    Ok(())
}


async fn fetch_workflow_runs(client: &MCPClient, owner: &str, repo: &str, pr_number: u64) -> Vec<WorkflowRun> {
    match client.get_pr_workflow_runs(owner, repo, pr_number).await {
        Ok(response) => {
            if let Some(runs) = response.get("workflow_runs").and_then(|r| r.as_array()) {
                runs.iter()
                    .filter_map(|run| serde_json::from_value(run.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

async fn fetch_check_runs(client: &MCPClient, owner: &str, repo: &str, sha: &str) -> Vec<CheckRun> {
    match client.get_pr_check_runs(owner, repo, sha).await {
        Ok(response) => {
            if let Some(checks) = response.get("check_runs").and_then(|c| c.as_array()) {
                checks.iter()
                    .filter_map(|check| serde_json::from_value(check.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

async fn fetch_status_checks(client: &MCPClient, owner: &str, repo: &str, sha: &str) -> Vec<StatusCheck> {
    match client.get_pr_status_checks(owner, repo, sha).await {
        Ok(response) => {
            if let Some(statuses) = response.get("statuses").and_then(|s| s.as_array()) {
                statuses.iter()
                    .filter_map(|status| serde_json::from_value(status.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

fn parse_repo(repo: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Repository must be in format 'owner/repo'"));
    }
    
    Ok((parts[0].to_string(), parts[1].to_string()))
}

async fn handle_status_check_command(
    client: &MCPClient,
    config: &Config,
    repos_path: &str,
    username: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let github_username = config.get_github_username()?;
    let username = username.unwrap_or(&github_username);
    let org_name = config.github_organization.as_deref().unwrap_or("organization");
    
    // Scan local repositories
    let local_repos = scan_local_repositories(repos_path)?;
    let mut all_prs = Vec::new();
    
    for local_repo in &local_repos {
        if let Some(ref org) = config.github_organization {
            if local_repo.owner != *org {
                continue;
            }
        }
        
        match client.list_pull_requests(&local_repo.owner, &local_repo.repo, "open", 10).await {
            Ok(pr_response) => {
                let prs: Vec<PullRequest> = pr_response
                    .into_iter()
                    .filter_map(|pr_json| serde_json::from_value(pr_json).ok())
                    .filter(|pr: &PullRequest| pr.user.login == username)
                    .collect();
                
                for pr in prs {
                    let workflow_status = get_pr_workflow_status(client, &local_repo.owner, &local_repo.repo, &pr).await;
                    let pr_json = PRStatusJson {
                        repository: format!("{}/{}", local_repo.owner, local_repo.repo),
                        number: pr.number,
                        title: pr.title.clone(),
                        author: pr.user.login.clone(),
                        status: workflow_status,
                        url: pr.html_url.clone(),
                        updated_at: pr.updated_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
                        is_draft: pr.is_draft(),
                    };
                    all_prs.push(pr_json);
                }
            }
            Err(_) => {
                // Skip repositories that fail to fetch
            }
        }
    }
    
    if json_output {
        let mut failing_prs = Vec::new();
        let mut running_prs = Vec::new();
        let mut passing_prs = Vec::new();
        let mut review_prs = Vec::new();
        
        for pr in all_prs.iter() {
            match pr.status.as_str() {
                "Build failing" | "Some checks failing" => failing_prs.push(pr.clone()),
                "Build running" => running_prs.push(pr.clone()),
                "Build passing" => passing_prs.push(pr.clone()),
                "No checks" => review_prs.push(pr.clone()), // PRs without CI/CD data
                _ => review_prs.push(pr.clone()), // Default unknown statuses
            }
        }
        
        let overall_status = if !failing_prs.is_empty() {
            "failing"
        } else if !running_prs.is_empty() {
            "running"
        } else if !review_prs.is_empty() {
            "needs_review"
        } else if !passing_prs.is_empty() {
            "passing"
        } else {
            "no_prs"
        };
        
        let summary = StatusSummaryJson {
            total_prs: all_prs.len(),
            failing_prs,
            running_prs,
            passing_prs,
            review_prs,
            overall_status: overall_status.to_string(),
            last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        // Regular human-readable output
        if all_prs.is_empty() {
            println!("📭 No open PRs found to monitor.");
        } else {
            println!("📊 Found {} open PRs for @{} in {} repositories", all_prs.len(), username, org_name);
            for pr in &all_prs {
                let status_emoji = match pr.status.as_str() {
                    "Build failing" => "❌",
                    "Some checks failing" => "⚠️",
                    "Build running" => "🔄",
                    "Build passing" => "✅",
                    _ => "❓",
                };
                println!("{} {} #{} - {}", status_emoji, pr.repository, pr.number, pr.title);
            }
        }
    }
    
    Ok(())
}

async fn handle_monitor_command(
    client: &MCPClient,
    config: &Config,
    repos_path: &str,
    interval: u64,
    username: Option<&str>,
    _json_output: bool,
) -> Result<()> {
    let github_username = config.get_github_username()?;
    let username = username.unwrap_or(&github_username);
    let org_name = config.github_organization.as_deref().unwrap_or("organization");
    
    println!("🔍 Starting PR monitor for @{} in {} repositories...", username, org_name);
    println!("📍 Scanning local repositories in: {}", repos_path);
    println!("⏱️  Check interval: {} seconds", interval);
    println!("🛑 Press Ctrl+C to stop monitoring\n");
    
    let mut previous_states: HashMap<String, String> = HashMap::new();
    let mut interval_timer = time::interval(Duration::from_secs(interval));
    
    loop {
        interval_timer.tick().await;
        
        println!("🔄 Checking PR status... ({})", chrono::Local::now().format("%H:%M:%S"));
        
        // Scan local repositories
        let local_repos = scan_local_repositories(repos_path)?;
        let mut prs_needing_attention = Vec::new();
        let mut all_monitored_prs = Vec::new();
        
        for local_repo in &local_repos {
            if let Some(ref org) = config.github_organization {
                if local_repo.owner != *org {
                    continue;
                }
            }
            
            match client.list_pull_requests(&local_repo.owner, &local_repo.repo, "open", 10).await {
                Ok(pr_response) => {
                    let prs: Vec<PullRequest> = pr_response
                        .into_iter()
                        .filter_map(|pr_json| serde_json::from_value(pr_json).ok())
                        .filter(|pr: &PullRequest| pr.user.login == username)
                        .collect();
                    
                    for pr in prs {
                        let pr_key = format!("{}/{}#{}", local_repo.owner, local_repo.repo, pr.number);
                        all_monitored_prs.push((pr_key.clone(), pr.clone()));
                        
                        // Get workflow status
                        let workflow_status = get_pr_workflow_status(client, &local_repo.owner, &local_repo.repo, &pr).await;
                        
                        // Check if this PR needs attention (CI/CD focused)
                        let needs_attention = match workflow_status.as_str() {
                            "Build failing" | "Some checks failing" => true, // Failing builds need immediate attention
                            "Build running" => true, // Monitor running builds for status changes
                            // "Build passing" and "No checks" don't need monitoring
                            _ => false,
                        };
                        
                        // Check for status changes
                        if let Some(previous_status) = previous_states.get(&pr_key) {
                            if previous_status != &workflow_status {
                                println!("📢 Status changed for {}: {} → {}", 
                                    pr_key, previous_status, workflow_status);
                            }
                        }
                        
                        // Update state tracking
                        previous_states.insert(pr_key.clone(), workflow_status.clone());
                        
                        if needs_attention {
                            prs_needing_attention.push((pr_key, pr, workflow_status));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to fetch PRs for {}/{}: {}", 
                        local_repo.owner, local_repo.repo, e);
                }
            }
        }
        
        // Display results
        if !prs_needing_attention.is_empty() {
            // Separate running from failing
            let mut failing_prs = Vec::new();
            let mut running_prs = Vec::new();
            
            for item in &prs_needing_attention {
                match item.2.as_str() {
                    "Build running" => running_prs.push(item),
                    _ => failing_prs.push(item),
                }
            }
            
            if !failing_prs.is_empty() {
                println!("\n🔴 PRs Needing Attention ({}):", failing_prs.len());
                println!("{}", "=".repeat(50));
                
                for (pr_key, pr, status) in &failing_prs {
                    let status_emoji = match status.as_str() {
                        "Build failing" => "❌",
                        "Some checks failing" => "⚠️",
                        _ => "🔴",
                    };
                    
                    println!("{} {} - {}", status_emoji, pr_key, pr.title);
                    println!("   Status: {} | Updated: {}", 
                        status, 
                        pr.updated_at.with_timezone(&chrono::Local).format("%H:%M:%S")
                    );
                    println!("   URL: {}", pr.html_url);
                    println!();
                }
            }
            
            if !running_prs.is_empty() {
                println!("\n🔄 PRs with Running Builds ({}):", running_prs.len());
                println!("{}", "=".repeat(50));
                
                for (pr_key, pr, status) in &running_prs {
                    println!("🔄 {} - {}", pr_key, pr.title);
                    println!("   Status: {} | Updated: {}", 
                        status, 
                        pr.updated_at.with_timezone(&chrono::Local).format("%H:%M:%S")
                    );
                    println!("   URL: {}", pr.html_url);
                    // Show currently running checks (GitHub Actions check runs)
                    if let Some((owner, repo)) = pr_key.split('#').next().and_then(|full| {
                        let parts: Vec<&str> = full.split('/').collect();
                        if parts.len() == 2 { Some((parts[0].to_string(), parts[1].to_string())) } else { None }
                    }) {
                        let running_checks = fetch_check_runs(client, &owner, &repo, &pr.head.sha).await;
                        let mut printed_any = false;
                        for check in running_checks.iter() {
                            if check.status == "in_progress" || check.status == "queued" || check.status == "requested" {
                                println!("   🔄 {} ({})", check.name, check.status);
                                printed_any = true;
                            }
                        }
                        if printed_any {
                            println!();
                        }
                    }
                    println!();
                }
            }
        } else if !all_monitored_prs.is_empty() {
            println!("✅ All {} monitored PRs are looking good!", all_monitored_prs.len());
        } else {
            println!("📭 No open PRs found to monitor.");
        }
        
        println!("⏰ Next check in {} seconds...\n", interval);
    }
}

async fn get_pr_workflow_status(
    client: &MCPClient,
    owner: &str,
    repo: &str,
    pr: &PullRequest,
) -> String {
    let sha = &pr.head.sha;
    
    // Fetch workflow runs, check runs, status checks, reviews, and combined status in parallel
    // Use commit-specific workflow runs for more accurate data
    let (workflow_runs_result, check_runs_result, status_checks_result, reviews_result, combined_status_result) = tokio::join!(
        client.get_workflow_runs_for_commit(owner, repo, sha),
        client.get_pr_check_runs(owner, repo, sha),
        client.get_pr_status_checks(owner, repo, sha),
        client.get_pr_reviews(owner, repo, pr.number),
        client.get_combined_status(owner, repo, sha)
    );
    
    let workflow_runs_json = workflow_runs_result.unwrap_or_else(|_| serde_json::json!({"workflow_runs": []}));
    let check_runs_json = check_runs_result.unwrap_or_else(|_| serde_json::json!({"check_runs": []}));
    let status_checks_json = status_checks_result.unwrap_or_else(|_| serde_json::json!([]));
    let _reviews_json = reviews_result.unwrap_or_else(|_| serde_json::json!([]));
    let _combined_status_json = combined_status_result.unwrap_or_else(|_| serde_json::json!({}));
    
    // Skip combined status API for now - it's for legacy status checks, not GitHub Actions
    // GitHub Actions use check runs, which we'll analyze below
    // The combined status API can be misleading when GitHub Actions are running
    
    // For GitHub Actions, we need to look at check runs to get the "official" status
    // Check if all required check runs are successful
    if let Some(check_runs) = check_runs_json.get("check_runs").and_then(|c| c.as_array()) {
        let mut has_required_failures = false;
        let mut has_required_pending = false;
        let mut has_any_required_checks = false;
        
        for check in check_runs {
            // Skip draft checks and non-required checks if possible
            if let Some(name) = check.get("name").and_then(|n| n.as_str()) {
                // Skip generic "Build" workflows - they're not meaningful
                if name == "Build" {
                    continue;
                }
                
                has_any_required_checks = true;
                
                if let Some(status) = check.get("status").and_then(|s| s.as_str()) {
                    match status {
                        "completed" => {
                            if let Some(conclusion) = check.get("conclusion").and_then(|c| c.as_str()) {
                                match conclusion {
                                    "failure" | "cancelled" | "timed_out" => {
                                        has_required_failures = true;
                                    }
                                    "skipped" | "success" => {
                                        // These are fine
                                    }
                                    _ => {
                                        // Unknown conclusion, treat as failure for safety
                                        has_required_failures = true;
                                    }
                                }
                            }
                        }
                        "in_progress" | "queued" | "requested" => {
                            has_required_pending = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // If we have required checks, use their status as the official status
        if has_any_required_checks {
            if has_required_pending {
                return "Build running".to_string();
            } else if has_required_failures {
                return "Build failing".to_string();
            } else {
                return "Build passing".to_string();
            }
        }
    }
    
    
    // Analyze workflow runs (excluding generic "Build" workflows)
    let mut has_failing_workflows = false;
    let mut has_running_workflows = false;
    
    if let Some(runs) = workflow_runs_json.get("workflow_runs").and_then(|r| r.as_array()) {
        for run in runs {
            // Skip generic "Build" workflows - they're not meaningful
            if let Some(name) = run.get("name").and_then(|n| n.as_str()) {
                if name == "Build" {
                    continue;
                }
            }
            
            if let Some(status) = run.get("status").and_then(|s| s.as_str()) {
                match status {
                    "completed" => {
                        if let Some(conclusion) = run.get("conclusion").and_then(|c| c.as_str()) {
                            match conclusion {
                                "failure" => has_failing_workflows = true,
                                "skipped" => {
                                    // Treat skipped workflows as successful
                                }
                                "success" => {
                                    // Explicitly successful
                                }
                                _ => {}
                            }
                        }
                    }
                    "in_progress" | "queued" => {
                        has_running_workflows = true;
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Analyze check runs
    let mut has_failing_checks = false;
    let mut has_running_checks = false;
    if let Some(checks) = check_runs_json.get("check_runs").and_then(|c| c.as_array()) {
        for check in checks {
            if let Some(status) = check.get("status").and_then(|s| s.as_str()) {
                match status {
                    "completed" => {
                        if let Some(conclusion) = check.get("conclusion").and_then(|c| c.as_str()) {
                            match conclusion {
                                "failure" => has_failing_checks = true,
                                "skipped" | "success" => {
                                    // Treat skipped and success as successful - don't set any failure flags
                                }
                                _ => {}
                            }
                        }
                    }
                    "in_progress" | "queued" | "requested" => {
                        has_running_checks = true;
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Analyze status checks
    let mut has_failing_status = false;
    let mut has_pending_status = false;
    if let Some(statuses) = status_checks_json.as_array() {
        for status in statuses {
            if let Some(state) = status.get("state").and_then(|s| s.as_str()) {
                match state {
                    "failure" | "error" => has_failing_status = true,
                    "pending" => has_pending_status = true,
                    "success" => {
                        // Explicitly successful
                    }
                    _ => {
                        // Treat unknown states (including skipped) as non-failing
                    }
                }
            }
        }
    }
    
    // Note: Review status is not considered in CI/CD focused mode
    // We only care about build/test status, not review requirements
    
    // Fallback: Determine status from individual checks if combined status is unavailable
    // 1. If CI/CD checks are still running -> "Build running" (highest priority - current state matters most)
    // 2. If NO running checks but some have failed -> "Build failing" (second priority)
    // 3. If CI/CD checks are complete and successful -> "Build passing" (third priority)
    // 4. If no CI/CD data available -> "No checks" (lowest priority)
    
    if has_running_workflows || has_running_checks {
        // GitHub Actions are still running - this is the current state and what we care about most
        "Build running".to_string()
    } else if has_failing_workflows || has_failing_checks || has_failing_status {
        // Only show as failing if nothing is currently running
        if has_failing_workflows {
            "Build failing".to_string()
        } else {
            "Some checks failing".to_string()
        }
    } else if !workflow_runs_json.get("workflow_runs").and_then(|r| r.as_array()).unwrap_or(&vec![]).is_empty() ||
              !check_runs_json.get("check_runs").and_then(|c| c.as_array()).unwrap_or(&vec![]).is_empty() {
        // We have GitHub Actions data and all are complete and successful
        // Ignore pending external status checks if GitHub Actions are done
        "Build passing".to_string()
    } else if has_pending_status {
        // Only external status checks exist and some are pending
        "Build running".to_string()
    } else {
        // No CI/CD data available - not much to monitor
        "No checks".to_string()
    }
}