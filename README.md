# PR Tracker CLI

A Rust-based CLI tool that uses MCP (Model Context Protocol) to list GitHub PR status in the terminal.

## Features

- **Defaults to YOUR PRs only** - Focus on what matters to you
- List GitHub pull requests with status information
- Get detailed status of specific pull requests
- List all repositories you have access to
- Scan PRs across multiple repositories at once
- **NEW**: Check local repositories for your PRs with CI/CD status
- **NEW**: Show detailed workflow/CI status for PRs
- Colorized terminal output with status indicators
- Support for different PR states (open, closed, all)
- Configurable output limits

## Installation

```bash
cargo build --release
```

## Usage

### Prerequisites

- GitHub personal access token
- Repository in format `owner/repo`

### Basic Usage

```bash
# List open PRs (default)
./target/release/prtracker --token YOUR_GITHUB_TOKEN --repo owner/repo list

# List closed PRs
./target/release/prtracker --token YOUR_GITHUB_TOKEN --repo owner/repo list --state closed

# List all PRs with custom limit
./target/release/prtracker --token YOUR_GITHUB_TOKEN --repo owner/repo list --state all --limit 20

# Get specific PR status
./target/release/prtracker --token YOUR_GITHUB_TOKEN --repo owner/repo status 123

# List your repositories
./target/release/prtracker --token YOUR_GITHUB_TOKEN repos

# Scan YOUR PRs across all your repositories (default behavior)
./target/release/prtracker --token YOUR_GITHUB_TOKEN scan

# Scan with custom settings (still shows only your PRs)
./target/release/prtracker --token YOUR_GITHUB_TOKEN scan --state all --limit 10 --max-repos 20

# Show ALL PRs (not just yours) across repositories
./target/release/prtracker --token YOUR_GITHUB_TOKEN scan --all

# Check YOUR PRs in local repositories with CI/CD status
./target/release/prtracker --token YOUR_GITHUB_TOKEN local --workflows

# Check ALL PRs in local repositories
./target/release/prtracker --token YOUR_GITHUB_TOKEN local --all

# Show only your PRs with detailed workflow status
./target/release/prtracker --token YOUR_GITHUB_TOKEN my-prs

# Monitor PRs for status changes (checks every 60 seconds)
./target/release/prtracker --token YOUR_GITHUB_TOKEN monitor

# Monitor with custom interval (checks every 30 seconds)
./target/release/prtracker --token YOUR_GITHUB_TOKEN monitor --interval 30
```

### Environment Variables

You can set environment variables to avoid passing tokens and repo every time:

```bash
export GITHUB_TOKEN="your_token_here"
export GITHUB_REPO="owner/repo"
```

Then run:
```bash
./target/release/prtracker list
```

## Commands

- `list` - List pull requests (requires --repo)
  - `--state` - Filter by state (open, closed, all) [default: open]
  - `--limit` - Number of PRs to display [default: 10]

- `status <pr_number>` - Get detailed status of a specific PR (requires --repo)

- `repos` - List repositories you have access to
  - `--limit` - Number of repos to display [default: 20]

- `scan` - Check YOUR PRs across multiple repositories (default behavior)
  - `--state` - Filter by state (open, closed, all) [default: open]
  - `--limit` - Number of PRs per repository [default: 5]
  - `--max-repos` - Maximum repositories to check [default: 10]
  - `--all` - Show all PRs, not just yours
  - `--username` - Specify GitHub username (auto-detected from git config)

- `local` - Check YOUR PRs in local repositories (default behavior)
  - `--state` - Filter by state (open, closed, all) [default: open]
  - `--limit` - Number of PRs per repository [default: 5]
  - `--path` - Path to repos directory [default: ~/repos]
  - `--workflows` - Show detailed CI/CD workflow status
  - `--all` - Show all PRs, not just yours
  - `--username` - Specify GitHub username (auto-detected from git config)

- `my-prs` - Show only your PRs with detailed workflow status
  - `--state` - Filter by state (open, closed, all) [default: open]
  - `--path` - Path to repos directory [default: ~/repos]
  - `--username` - Specify GitHub username (auto-detected from git config)

- `monitor` - Continuously monitor PRs with running workflows and alert on status changes
  - `--path` - Path to repos directory [default: ~/repos]
  - `--interval` - Check interval in seconds [default: 60]
  - `--username` - Specify GitHub username (auto-detected from git config)
  - Press Ctrl+C to stop monitoring

## Output

The tool provides colorized output with:
- 🟢 Open PRs
- 🔴 Closed PRs  
- 🟣 Merged PRs
- ✅ Clean mergeable state
- ⚠️ Dirty mergeable state
- ❌ Unstable mergeable state
- 🚫 Blocked mergeable state
- [DRAFT] for draft PRs

### CI/CD Status Indicators (with --workflows flag)
- ✅ Successful checks/workflows
- ❌ Failed checks/workflows  
- 🔄 In progress
- ⏳ Queued
- ⚪ Neutral/skipped
- ⏭️ Cancelled
- ❓ Unknown status

## Development

```bash
# Check code
cargo check

# Run tests
cargo test

# Build
cargo build

# Run with debug output
RUST_LOG=debug cargo run -- --token YOUR_TOKEN --repo owner/repo list
```
