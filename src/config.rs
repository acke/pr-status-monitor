use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// GitHub token for authentication
    pub github_token: Option<String>,
    
    /// GitHub username
    pub github_username: Option<String>,
    
    /// Base GitHub URL (default: https://api.github.com)
    pub github_base_url: String,
    
    /// Local repositories folder path
    pub local_repos_path: String,
    
    /// GitHub organization to filter by (e.g., "snyk")
    pub github_organization: Option<String>,
    
    /// Default PR state to show (open, closed, all)
    pub default_pr_state: String,
    
    /// Default number of PRs to show
    pub default_pr_limit: usize,
    
    /// Default number of repositories to scan
    pub default_repo_limit: usize,
    
    /// Show workflows by default
    pub show_workflows_by_default: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github_token: None,
            github_username: None,
            github_base_url: "https://api.github.com".to_string(),
            local_repos_path: "~/repos".to_string(),
            github_organization: Some("snyk".to_string()),
            default_pr_state: "open".to_string(),
            default_pr_limit: 10,
            default_repo_limit: 20,
            show_workflows_by_default: false,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?;
        
        let app_config_dir = config_dir.join("my-prs");
        fs::create_dir_all(&app_config_dir)?;
        
        Ok(app_config_dir.join("config.toml"))
    }
    
    /// Load config from file, or create default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = Config::default();
            default_config.save()?;
            Ok(default_config)
        }
    }
    
    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        println!("✅ Configuration saved to: {}", config_path.display());
        Ok(())
    }
    
    /// Run interactive configuration wizard
    pub fn run_wizard() -> Result<Self> {
        println!("🔧 My PRs Configuration Wizard");
        println!("===================================\n");
        
        let mut config = Config::default();
        
        // GitHub Token
        print!("GitHub Token (leave empty to use GITHUB_PRIVATE_TOKEN env var): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let token = input.trim();
        if !token.is_empty() {
            config.github_token = Some(token.to_string());
        }
        
        // GitHub Username
        print!("GitHub Username (leave empty for auto-detection): ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let username = input.trim();
        if !username.is_empty() {
            config.github_username = Some(username.to_string());
        }
        
        // GitHub Base URL
        print!("GitHub API Base URL [{}]: ", config.github_base_url);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let base_url = input.trim();
        if !base_url.is_empty() {
            config.github_base_url = base_url.to_string();
        }
        
        // Local Repos Path
        print!("Local repositories path [{}]: ", config.local_repos_path);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let repos_path = input.trim();
        if !repos_path.is_empty() {
            config.local_repos_path = repos_path.to_string();
        }
        
        // GitHub Organization
        let default_org = config.github_organization.as_deref().unwrap_or("none");
        print!("GitHub Organization to filter by [{}]: ", default_org);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let org = input.trim();
        if !org.is_empty() {
            if org.to_lowercase() == "none" {
                config.github_organization = None;
            } else {
                config.github_organization = Some(org.to_string());
            }
        }
        
        // Default PR State
        print!("Default PR state (open/closed/all) [{}]: ", config.default_pr_state);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let state = input.trim();
        if !state.is_empty() && ["open", "closed", "all"].contains(&state) {
            config.default_pr_state = state.to_string();
        }
        
        // Default PR Limit
        print!("Default number of PRs to show [{}]: ", config.default_pr_limit);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let limit = input.trim();
        if !limit.is_empty() {
            if let Ok(num) = limit.parse::<usize>() {
                config.default_pr_limit = num;
            }
        }
        
        // Default Repo Limit
        print!("Default number of repositories to scan [{}]: ", config.default_repo_limit);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let repo_limit = input.trim();
        if !repo_limit.is_empty() {
            if let Ok(num) = repo_limit.parse::<usize>() {
                config.default_repo_limit = num;
            }
        }
        
        // Show Workflows by Default
        print!("Show workflows by default? (y/n) [{}]: ", if config.show_workflows_by_default { "y" } else { "n" });
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let workflows = input.trim().to_lowercase();
        if workflows == "y" || workflows == "yes" {
            config.show_workflows_by_default = true;
        } else if workflows == "n" || workflows == "no" {
            config.show_workflows_by_default = false;
        }
        
        println!("\n📋 Configuration Summary:");
        println!("========================");
        println!("GitHub Token: {}", if config.github_token.is_some() { "Set" } else { "Use GITHUB_PRIVATE_TOKEN env var" });
        println!("GitHub Username: {}", config.github_username.as_deref().unwrap_or("Auto-detect"));
        println!("GitHub Base URL: {}", config.github_base_url);
        println!("Local Repos Path: {}", config.local_repos_path);
        println!("GitHub Organization: {}", config.github_organization.as_deref().unwrap_or("None"));
        println!("Default PR State: {}", config.default_pr_state);
        println!("Default PR Limit: {}", config.default_pr_limit);
        println!("Default Repo Limit: {}", config.default_repo_limit);
        println!("Show Workflows by Default: {}", config.show_workflows_by_default);
        
        print!("\nSave this configuration? (y/n): ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let save = input.trim().to_lowercase();
        
        if save == "y" || save == "yes" {
            config.save()?;
        } else {
            println!("❌ Configuration not saved.");
        }
        
        Ok(config)
    }
    
    /// Get the effective GitHub token (from config or environment)
    pub fn get_github_token(&self) -> Option<String> {
        self.github_token.clone()
            .or_else(|| std::env::var("GITHUB_PRIVATE_TOKEN").ok())
            .or_else(|| std::env::var("GITHUB_TOKEN").ok())
    }
    
    /// Get the effective GitHub username
    pub fn get_github_username(&self) -> Result<String> {
        if let Some(username) = &self.github_username {
            return Ok(username.clone());
        }
        
        // Try to get from git config and map known display names
        let output = std::process::Command::new("git")
            .args(["config", "--global", "user.name"])
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let git_name = String::from_utf8(output.stdout)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                
                // Map display name to GitHub username
                match git_name.as_str() {
                    "Knut Funkel" => Ok("acke".to_string()),
                    name if !name.is_empty() => Ok(name.to_string()),
                    _ => Ok("acke".to_string()), // Default fallback
                }
            }
            _ => Ok("acke".to_string()), // Default fallback if git config fails
        }
    }
    
    /// Expand tilde in path
    pub fn expand_path(&self, path: &str) -> String {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                return home_dir.join(stripped).to_string_lossy().to_string();
            }
        }
        path.to_string()
    }
    
    /// Get the expanded local repos path
    pub fn get_local_repos_path(&self) -> String {
        self.expand_path(&self.local_repos_path)
    }
}
