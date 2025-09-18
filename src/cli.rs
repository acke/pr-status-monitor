use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "my-prs")]
#[command(about = "A CLI tool to track your GitHub PR status")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// GitHub token for authentication
    #[arg(short, long, env("GITHUB_PRIVATE_TOKEN"))]
    pub token: Option<String>,
    
    /// GitHub repository in format owner/repo
    #[arg(short, long)]
    pub repo: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List your pull requests in a specific repository
    List {
        /// Filter by PR state (open, closed, all)
        #[arg(short, long)]
        state: Option<String>,
        
        /// Number of PRs to display
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Get status of all open PRs (or specific PR if number provided)
    Status {
        /// PR number (optional - shows all open PRs if not provided)
        pr_number: Option<u64>,
    },
    /// List repositories you have access to
    Repos {
        /// Number of repos to display
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Check PRs across multiple repositories (defaults to your PRs only)
    Scan {
        /// Filter by PR state (open, closed, all)
        #[arg(short, long)]
        state: Option<String>,
        
        /// Number of PRs per repository
        #[arg(short, long)]
        limit: Option<usize>,
        
        /// Maximum number of repositories to check
        #[arg(short, long)]
        max_repos: Option<usize>,
        
        /// Show all PRs, not just yours
        #[arg(short, long)]
        all: bool,
        
        /// Your GitHub username (auto-detected from git config if not provided)
        #[arg(short, long)]
        username: Option<String>,
    },
    /// Check PRs for local repositories in ~/repos/ (defaults to your PRs only)
    Local {
        /// Filter by PR state (open, closed, all)
        #[arg(short, long)]
        state: Option<String>,
        
        /// Number of PRs per repository
        #[arg(short, long)]
        limit: Option<usize>,
        
        /// Path to repos directory
        #[arg(short, long)]
        path: Option<String>,
        
        /// Show workflow/CI status for PRs
        #[arg(short, long)]
        workflows: Option<bool>,
        
        /// Show all PRs, not just yours
        #[arg(short, long)]
        all: bool,
        
        /// Your GitHub username (auto-detected from git config if not provided)
        #[arg(short, long)]
        username: Option<String>,
    },
    /// Show only your PRs with workflow status
    MyPrs {
        /// Filter by PR state (open, closed, all)
        #[arg(short, long)]
        state: Option<String>,
        
        /// Path to repos directory
        #[arg(short, long)]
        path: Option<String>,
        
        /// Your GitHub username (auto-detected from git config if not provided)
        #[arg(short, long)]
        username: Option<String>,
    },
    /// Configure my-prs settings
    Config {
        /// Run interactive configuration wizard
        #[arg(long)]
        wizard: bool,
        
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
    
    /// Monitor PRs with running workflows and alert on status changes
    Monitor {
        /// Path to local repositories (default from config)
        #[arg(short, long)]
        path: Option<String>,
        
        /// Check interval in seconds (default: 60)
        #[arg(short, long, default_value = "60")]
        interval: u64,
        
        /// Username to filter PRs (default: your username)
        #[arg(short, long)]
        username: Option<String>,
        
        /// Output JSON for machine consumption
        #[arg(long)]
        json: bool,
    },
    
    /// Get current PR status (single check, no monitoring)
    StatusCheck {
        /// Path to local repositories (default from config)
        #[arg(short, long)]
        path: Option<String>,
        
        /// Username to filter PRs (default: your username)
        #[arg(short, long)]
        username: Option<String>,
        
        /// Output JSON for machine consumption
        #[arg(long)]
        json: bool,
    },
}
