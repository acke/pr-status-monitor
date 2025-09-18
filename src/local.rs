use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct LocalRepo {
    pub name: String,
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub remote_url: String,
    pub owner: String,
    pub repo: String,
}

pub fn scan_local_repositories(repos_path: &str) -> Result<Vec<LocalRepo>> {
    let expanded_path = expand_tilde(repos_path);
    let repos_dir = Path::new(&expanded_path);
    
    if !repos_dir.exists() {
        return Err(anyhow!("Directory {} does not exist", expanded_path));
    }
    
    let mut local_repos = Vec::new();
    
    // Read all directories in the repos folder
    let entries = fs::read_dir(repos_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            // Skip hidden directories and files
            if dir_name.starts_with('.') {
                continue;
            }
            
            // Check if it's a git repository
            let git_dir = path.join(".git");
            if git_dir.exists() {
                if let Ok(remote_url) = get_git_remote_url(&path) {
                    if let Ok((owner, repo)) = parse_github_url(&remote_url) {
                        local_repos.push(LocalRepo {
                            name: dir_name.to_string(),
                            path: path.to_string_lossy().to_string(),
                            remote_url,
                            owner,
                            repo,
                        });
                    }
                }
            }
        }
    }
    
    Ok(local_repos)
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return format!("{}{}", home.to_string_lossy(), &path[1..]);
        }
    }
    path.to_string()
}

fn get_git_remote_url(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow!("Failed to get git remote URL"));
    }
    
    let url = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    
    Ok(url)
}

fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Handle both SSH and HTTPS URLs
    let cleaned_url = if url.starts_with("git@github.com:") {
        // SSH format: git@github.com:owner/repo.git
        url.strip_prefix("git@github.com:")
            .unwrap_or(url)
    } else if url.starts_with("https://github.com/") {
        // HTTPS format: https://github.com/owner/repo.git
        url.strip_prefix("https://github.com/")
            .unwrap_or(url)
    } else {
        return Err(anyhow!("Not a GitHub URL: {}", url));
    };
    
    // Remove .git suffix if present
    let cleaned_url = cleaned_url.strip_suffix(".git").unwrap_or(cleaned_url);
    
    // Split into owner/repo
    let parts: Vec<&str> = cleaned_url.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid GitHub URL format: {}", url));
    }
    
    Ok((parts[0].to_string(), parts[1].to_string()))
}
