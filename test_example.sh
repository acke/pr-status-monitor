#!/bin/bash

# Example usage of the PR Tracker CLI
# This script demonstrates how to use the CLI with example commands

echo "=== PR Tracker CLI Example ==="
echo

echo "1. Show help:"
echo "./target/debug/prtracker --help"
echo

echo "2. Show list command help:"
echo "./target/debug/prtracker list --help"
echo

echo "3. Example commands (replace with your actual token and repo):"
echo "export GITHUB_TOKEN='your_github_token_here'"
echo "export GITHUB_REPO='owner/repository'"
echo
echo "# List open PRs"
echo "./target/debug/prtracker list"
echo
echo "# List closed PRs"
echo "./target/debug/prtracker list --state closed"
echo
echo "# List all PRs with limit"
echo "./target/debug/prtracker list --state all --limit 20"
echo
echo "# Get specific PR status"
echo "./target/debug/prtracker status 123"
echo
echo "# List your repositories"
echo "./target/debug/prtracker repos"
echo
echo "# Scan YOUR PRs across all your repositories (default behavior)"
echo "./target/debug/prtracker scan"
echo
echo "# Scan with custom settings (still shows only your PRs)"
echo "./target/debug/prtracker scan --state all --limit 10 --max-repos 20"
echo
echo "# Show ALL PRs (not just yours) across repositories"
echo "./target/debug/prtracker scan --all"
echo
echo "# Check YOUR PRs in local repositories with CI/CD status"
echo "./target/debug/prtracker local --workflows"
echo
echo "# Check ALL PRs in local repositories"
echo "./target/debug/prtracker local --all"
echo
echo "# Show only your PRs with detailed workflow status"
echo "./target/debug/prtracker my-prs"
echo

echo "=== Build Instructions ==="
echo "cargo build --release"
echo "cp target/release/prtracker /usr/local/bin/  # Optional: install globally"
