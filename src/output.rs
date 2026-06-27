use crate::github::{PullRequest, Repository, WorkflowRun, CheckRun, StatusCheck};
use colored::*;

// Type aliases for complex types to improve readability
type PrWithChecks = (PullRequest, Vec<WorkflowRun>, Vec<CheckRun>, Vec<StatusCheck>);
type PrsByRepo = [(String, Vec<PrWithChecks>)];

pub fn print_pr_list(prs: &[PullRequest]) {
    if prs.is_empty() {
        println!("{}", "No pull requests found.".yellow());
        return;
    }

    println!("{}", "GitHub Pull Requests".bold().blue());
    println!("{}", "=".repeat(50).blue());
    
    for pr in prs {
        print_pr_summary(pr);
        println!();
    }
}

pub fn print_pr_summary(pr: &PullRequest) {
    let status_emoji = pr.status_emoji();
    let mergeable_emoji = pr.mergeable_emoji();
    let draft_indicator = if pr.is_draft() { " [DRAFT]" } else { "" };
    
    println!(
        "{} #{} {} {}{}",
        status_emoji,
        pr.number.to_string().bold(),
        truncate_string(&pr.title, 60),
        mergeable_emoji,
        draft_indicator.red()
    );
    
    println!(
        "   {} • {} • {}",
        format!("@{}", pr.user.login).dimmed(),
        pr.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string().dimmed(),
        pr.state.to_uppercase().color(match pr.state.as_str() {
            "open" => Color::Green,
            "closed" => Color::Red,
            "merged" => Color::Magenta,
            _ => Color::White,
        })
    );
    
    println!("   {}", pr.html_url.dimmed());
}

pub fn print_pr_details(pr: &PullRequest) {
    println!("{}", "Pull Request Details".bold().blue());
    println!("{}", "=".repeat(50).blue());
    
    let status_emoji = pr.status_emoji();
    let mergeable_emoji = pr.mergeable_emoji();
    let draft_indicator = if pr.is_draft() { " [DRAFT]" } else { "" };
    
    println!("{} #{} {}{}", status_emoji, pr.number, pr.title, draft_indicator.red());
    println!();
    
    println!("{}: {}", "State".bold(), pr.state.to_uppercase());
    println!("{}: @{}", "Author".bold(), pr.user.login);
    println!("{}: {}", "Created".bold(), pr.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S"));
    println!("{}: {}", "Updated".bold(), pr.updated_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S"));
    println!("{}: {}", "From".bold(), pr.head.label);
    println!("{}: {}", "To".bold(), pr.base.label);
    
    if let Some(mergeable) = pr.mergeable {
        println!("{}: {} {}", "Mergeable".bold(), mergeable_emoji, mergeable);
    }
    
    if let Some(state) = &pr.mergeable_state {
        println!("{}: {}", "Merge State".bold(), state);
    }
    
    println!();
    println!("{}: {}", "URL".bold(), pr.html_url);
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

pub fn print_repository_list(repos: &[Repository]) {
    if repos.is_empty() {
        println!("{}", "No repositories found.".yellow());
        return;
    }

    println!("{}", "Your Repositories".bold().blue());
    println!("{}", "=".repeat(50).blue());
    
    for repo in repos {
        print_repository_summary(repo);
        println!();
    }
}

pub fn print_repository_summary(repo: &Repository) {
    let private_indicator = if repo.private { "🔒" } else { "🌐" };
    let language = repo.language.as_deref().unwrap_or("Unknown");
    
    println!(
        "{} {} {}",
        private_indicator,
        repo.full_name.bold(),
        truncate_string(repo.description.as_deref().unwrap_or("No description"), 50)
    );
    
    println!(
        "   {} • {} • {} • {}",
        language.dimmed(),
        format!("⭐ {}", repo.stargazers_count).dimmed(),
        format!("🍴 {}", repo.forks_count).dimmed(),
        repo.updated_at.with_timezone(&chrono::Local).format("%Y-%m-%d").to_string().dimmed()
    );
    
    println!("   {}", repo.html_url.dimmed());
}

pub fn print_multi_repo_prs(prs_by_repo: &[(String, Vec<PullRequest>)]) {
    if prs_by_repo.is_empty() {
        println!("{}", "No pull requests found across repositories.".yellow());
        return;
    }

    println!("{}", "Pull Requests Across Repositories".bold().blue());
    println!("{}", "=".repeat(60).blue());
    
    for (repo_name, prs) in prs_by_repo {
        if !prs.is_empty() {
            println!("\n{}", format!("📁 {}", repo_name).bold().cyan());
            println!("{}", "-".repeat(repo_name.len() + 4).cyan());
            
            for pr in prs {
                print_pr_summary(pr);
            }
        }
    }
}

pub fn print_workflow_status(workflow_runs: &[WorkflowRun], check_runs: &[CheckRun], status_checks: &[StatusCheck]) {
    if workflow_runs.is_empty() && check_runs.is_empty() && status_checks.is_empty() {
        println!("      {}", "No CI/CD status available".dimmed());
        return;
    }

    // Print workflow runs (excluding generic "Build" workflows)
    for run in workflow_runs {
        // Skip generic "Build" workflows - they're not meaningful
        if run.name == "Build" {
            continue;
        }
        
        let status_emoji = match run.conclusion.as_deref() {
            Some("success") => "✅",
            Some("failure") => "❌",
            Some("cancelled") => "⚪",
            Some("skipped") => "⏭️",
            None if run.status == "in_progress" => "🔄",
            None if run.status == "queued" => "⏳",
            _ => "❓",
        };
        
        println!("      {} {} ({})", 
                status_emoji, 
                run.name.bold(), 
                run.status.dimmed());
    }

    // Print check runs
    for check in check_runs {
        let status_emoji = match check.conclusion.as_deref() {
            Some("success") => "✅",
            Some("failure") => "❌",
            Some("cancelled") => "⚪",
            Some("skipped") => "⏭️",
            None if check.status == "in_progress" => "🔄",
            None if check.status == "queued" => "⏳",
            _ => "❓",
        };
        
        println!("      {} {} ({})", 
                status_emoji, 
                check.name.bold(), 
                check.status.dimmed());
    }

    // Print status checks
    for status in status_checks {
        let status_emoji = match status.state.as_str() {
            "success" => "✅",
            "failure" => "❌",
            "error" => "🔴",
            "pending" => "🔄",
            _ => "❓",
        };
        
        let description = status.description.as_deref().unwrap_or("No description");
        println!("      {} {} - {}", 
                status_emoji, 
                status.context.bold(), 
                description.dimmed());
    }
}

pub fn get_overall_ci_status(workflow_runs: &[WorkflowRun], check_runs: &[CheckRun], status_checks: &[StatusCheck]) -> &'static str {
    // Filter out generic "Build" workflows
    let meaningful_workflows: Vec<&WorkflowRun> = workflow_runs.iter().filter(|w| w.name != "Build").collect();
    
    // Check if any are still running (highest priority - current state matters most)
    if meaningful_workflows.iter().any(|w| w.status == "in_progress" || w.status == "queued") ||
       check_runs.iter().any(|c| c.status == "in_progress" || c.status == "queued") ||
       status_checks.iter().any(|s| s.state == "pending") {
        return "🔄";
    }
    
    // Only check for failures if nothing is running
    if meaningful_workflows.iter().any(|w| w.conclusion.as_deref() == Some("failure")) {
        return "❌";
    }
    
    // Check if any check run failed
    if check_runs.iter().any(|c| c.conclusion.as_deref() == Some("failure")) {
        return "❌";
    }
    
    // Check if any status check failed
    if status_checks.iter().any(|s| s.state == "failure" || s.state == "error") {
        return "❌";
    }
    
    // Check if all completed successfully (treating skipped as successful)
    let all_workflows_success = meaningful_workflows.iter().all(|w| {
        matches!(w.conclusion.as_deref(), Some("success") | Some("skipped"))
    });
    let all_checks_success = check_runs.iter().all(|c| {
        matches!(c.conclusion.as_deref(), Some("success") | Some("skipped"))
    });
    let all_status_success = status_checks.iter().all(|s| s.state == "success");
    
    if all_workflows_success && all_checks_success && all_status_success && 
       (!meaningful_workflows.is_empty() || !check_runs.is_empty() || !status_checks.is_empty()) {
        return "✅";
    }
    
    "❓"
}

pub fn print_my_prs_with_workflow_status(
    prs_by_repo: &PrsByRepo,
    username: &str,
    org_name: &str,
) {
    if prs_by_repo.is_empty() {
        println!("{}", format!("No pull requests found for @{}", username).yellow());
        return;
    }

    println!("{}", format!("Your Pull Requests (@{})", username).bold().blue());
    println!("{}", "=".repeat(60).blue());
    
    let total_prs: usize = prs_by_repo.iter().map(|(_, prs)| prs.len()).sum();
    let total_repos = prs_by_repo.len();
    
    println!("📊 Summary: {} PRs across {} repositories\n", total_prs, total_repos);
    
    for (repo_name, prs_with_status) in prs_by_repo {
        println!("{}", format!("📁 {}", repo_name).bold().cyan());
        println!("{}", "-".repeat(repo_name.len() + 4).cyan());
        
        for (pr, workflow_runs, check_runs, status_checks) in prs_with_status {
            let status_emoji = pr.status_emoji();
            let mergeable_emoji = pr.mergeable_emoji();
            let draft_indicator = if pr.is_draft() { " [DRAFT]" } else { "" };
            let ci_status = get_overall_ci_status(workflow_runs, check_runs, status_checks);
            
            println!(
                "{} #{} {} {} {}{}",
                status_emoji,
                pr.number.to_string().bold(),
                truncate_string(&pr.title, 50),
                mergeable_emoji,
                ci_status,
                draft_indicator.red()
            );
            
            println!(
                "   {} • {} • {}",
                format!("@{}", pr.user.login).dimmed(),
                pr.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string().dimmed(),
                pr.state.to_uppercase().color(match pr.state.as_str() {
                    "open" => Color::Green,
                    "closed" => Color::Red,
                    "merged" => Color::Magenta,
                    _ => Color::White,
                })
            );
            
            // Show workflow status details
            if !workflow_runs.is_empty() || !check_runs.is_empty() || !status_checks.is_empty() {
                println!("   🔧 CI/CD Status:");
                print_workflow_status(workflow_runs, check_runs, status_checks);
            } else {
                println!("      {}", "No CI/CD status available".dimmed());
            }
            
            println!("   {}", pr.html_url.dimmed());
            println!();
        }
    }
    
    // Print detailed status summary at the end
    println!("{}", "=".repeat(60).blue());
    print_pr_status_summary(prs_by_repo, total_prs, total_repos, org_name);
}

fn print_pr_status_summary(
    prs_by_repo: &PrsByRepo,
    total_prs: usize,
    total_repos: usize,
    org_name: &str,
) {
    println!("📊 Your Current PR Status:");
    println!("You have {} open PRs across {} {} repositories:\n", total_prs, total_repos, org_name);
    
    let mut needs_attention = Vec::new();
    let mut looking_good = Vec::new();
    let mut draft_count = 0;
    
    // Analyze each PR
    for (repo_name, prs_with_status) in prs_by_repo {
        for (pr, workflow_runs, check_runs, status_checks) in prs_with_status {
            if pr.is_draft() {
                draft_count += 1;
            }
            
            let ci_status = get_overall_ci_status(workflow_runs, check_runs, status_checks);
            let has_failures = ci_status.contains("❌");
            let has_running = ci_status.contains("🔄");
            
            let status_description = if has_failures {
                get_failure_reason(workflow_runs, check_runs, status_checks)
            } else if has_running {
                get_running_reason(workflow_runs, check_runs, status_checks)
            } else {
                "All checks passing".to_string()
            };
            
            let pr_summary = format!("{} #{} - {}", repo_name, pr.number, status_description);
            
            if has_failures || has_running {
                needs_attention.push(pr_summary);
            } else {
                looking_good.push(pr_summary);
            }
        }
    }
    
    // Print needs attention section (failures and running)
    if !needs_attention.is_empty() {
        println!("🔴 Needs Attention:");
        for item in needs_attention {
            println!("   • {}", item);
        }
        println!();
    }
    
    // Print looking good section (only truly passing)
    if !looking_good.is_empty() {
        println!("🟢 Looking Good:");
        for item in looking_good {
            println!("   • {} ✅", item);
        }
        println!();
    }
    
    // Print draft note if applicable
    if draft_count > 0 {
        let draft_text = if draft_count == 1 { "PR is" } else { "PRs are" };
        println!("📝 Note:");
        if draft_count == total_prs {
            println!("   All your {} in DRAFT status, so they're works in progress.", draft_text);
        } else {
            println!("   {} of your {} in DRAFT status, so they're works in progress.", draft_count, draft_text);
        }
        println!();
    }
}

fn get_failure_reason(
    workflow_runs: &[WorkflowRun],
    check_runs: &[CheckRun],
    _status_checks: &[StatusCheck],
) -> String {
    // Check for specific failure patterns (excluding generic "Build" workflows)
    for run in workflow_runs {
        // Skip generic "Build" workflows - they're not meaningful
        if run.name == "Build" {
            continue;
        }
        
        if run.conclusion.as_deref() == Some("failure") {
            if run.name.to_lowercase().contains("integration") {
                if run.name.to_lowercase().contains("macos") {
                    return "macOS integration tests failing".to_string();
                } else {
                    return "Integration tests failing".to_string();
                }
            } else if run.name.to_lowercase().contains("test") {
                return "Tests failing".to_string();
            } else if run.name.to_lowercase().contains("build") {
                return "Build failing".to_string();
            } else if run.name.to_lowercase().contains("lint") {
                return "Linting issues".to_string();
            }
        }
    }
    
    for check in check_runs {
        if check.conclusion.as_deref() == Some("failure") {
            if check.name.to_lowercase().contains("integration") {
                return "Integration tests failing".to_string();
            } else if check.name.to_lowercase().contains("test") {
                return "Tests failing".to_string();
            } else if check.name.to_lowercase().contains("build") {
                return "Build failing".to_string();
            }
        }
    }
    
    "Some checks failing".to_string()
}

fn get_running_reason(
    workflow_runs: &[WorkflowRun],
    check_runs: &[CheckRun],
    status_checks: &[StatusCheck],
) -> String {
    // Check for specific running patterns (excluding generic "Build" workflows)
    for run in workflow_runs {
        // Skip generic "Build" workflows - they're not meaningful
        if run.name == "Build" {
            continue;
        }
        
        if run.status == "in_progress" || run.status == "queued" {
            if run.name.to_lowercase().contains("integration") {
                if run.name.to_lowercase().contains("macos") {
                    return "macOS integration tests running".to_string();
                } else {
                    return "Integration tests running".to_string();
                }
            } else if run.name.to_lowercase().contains("test") {
                return "Tests running".to_string();
            } else if run.name.to_lowercase().contains("build") {
                return "Build running".to_string();
            } else if run.name.to_lowercase().contains("lint") {
                return "Linting in progress".to_string();
            } else {
                return format!("{} running", run.name);
            }
        }
    }
    
    for check in check_runs {
        if check.status == "in_progress" || check.status == "queued" {
            if check.name.to_lowercase().contains("integration") {
                return "Integration tests running".to_string();
            } else if check.name.to_lowercase().contains("test") {
                return "Tests running".to_string();
            } else if check.name.to_lowercase().contains("build") {
                return "Build running".to_string();
            } else {
                return format!("{} running", check.name);
            }
        }
    }
    
    for status in status_checks {
        if status.state == "pending" {
            return format!("{} pending", status.context);
        }
    }
    
    "Checks running".to_string()
}

pub fn print_pr_status_overview(prs: &[PullRequest], _username: &str, org_name: &str) {
    if prs.is_empty() {
        return;
    }
    
    println!("{}", "=".repeat(60).blue());
    println!("📊 Your Current PR Status:");
    println!("You have {} open PRs in {} repositories:\n", prs.len(), org_name);
    
    let mut draft_count = 0;
    let mut ready_count = 0;
    
    for pr in prs {
        if pr.is_draft() {
            draft_count += 1;
        } else {
            ready_count += 1;
        }
    }
    
    if ready_count > 0 {
        println!("🟢 Ready for Review:");
        for pr in prs {
            if !pr.is_draft() {
                println!("   • #{} - {}", 
                    pr.number, 
                    truncate_string(&pr.title, 60)
                );
            }
        }
        println!();
    }
    
    if draft_count > 0 {
        println!("📝 Draft PRs:");
        for pr in prs {
            if pr.is_draft() {
                println!("   • #{} - {}", 
                    pr.number, 
                    truncate_string(&pr.title, 60)
                );
            }
        }
        println!();
        
        let draft_text = if draft_count == 1 { "PR is" } else { "PRs are" };
        println!("📝 Note:");
        println!("   {} of your {} in DRAFT status, so they're works in progress.", draft_count, draft_text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{CheckRun, StatusCheck, WorkflowRun};
    use chrono::Utc;

    fn workflow(name: &str, status: &str, conclusion: Option<&str>) -> WorkflowRun {
        WorkflowRun {
            id: 1,
            name: name.to_string(),
            status: status.to_string(),
            conclusion: conclusion.map(str::to_string),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            html_url: String::new(),
            workflow_id: 1,
        }
    }

    fn check(name: &str, status: &str, conclusion: Option<&str>) -> CheckRun {
        CheckRun {
            id: 1,
            name: name.to_string(),
            status: status.to_string(),
            conclusion: conclusion.map(str::to_string),
            started_at: None,
            completed_at: None,
            html_url: String::new(),
        }
    }

    fn status_check(state: &str) -> StatusCheck {
        StatusCheck {
            state: state.to_string(),
            description: None,
            context: "ci/test".to_string(),
            target_url: None,
        }
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate_string("abcdefghij", 7);
        assert_eq!(result, "abcd...");
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate_string("hello", 5), "hello");
    }

    #[test]
    fn test_ci_status_empty() {
        assert_eq!(get_overall_ci_status(&[], &[], &[]), "❓");
    }

    #[test]
    fn test_ci_status_all_success() {
        let wf = workflow("Tests", "completed", Some("success"));
        assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "✅");
    }

    #[test]
    fn test_ci_status_failure() {
        let wf = workflow("Tests", "completed", Some("failure"));
        assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "❌");
    }

    #[test]
    fn test_ci_status_in_progress() {
        let wf = workflow("Tests", "in_progress", None);
        assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "🔄");
    }

    #[test]
    fn test_ci_status_queued() {
        let wf = workflow("Tests", "queued", None);
        assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "🔄");
    }

    #[test]
    fn test_ci_status_skipped_counts_as_success() {
        let wf = workflow("Tests", "completed", Some("skipped"));
        assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "✅");
    }

    #[test]
    fn test_ci_status_build_workflow_ignored() {
        let build = workflow("Build", "completed", Some("failure"));
        assert_eq!(get_overall_ci_status(&[build], &[], &[]), "❓");
    }

    #[test]
    fn test_ci_status_running_takes_priority_over_failure() {
        let failing = workflow("Tests", "completed", Some("failure"));
        let running = check("lint", "in_progress", None);
        assert_eq!(get_overall_ci_status(&[failing], &[running], &[]), "🔄");
    }

    #[test]
    fn test_ci_status_check_run_failure() {
        let c = check("lint", "completed", Some("failure"));
        assert_eq!(get_overall_ci_status(&[], &[c], &[]), "❌");
    }

    #[test]
    fn test_ci_status_status_check_pending() {
        let s = status_check("pending");
        assert_eq!(get_overall_ci_status(&[], &[], &[s]), "🔄");
    }

    #[test]
    fn test_ci_status_status_check_failure() {
        let s = status_check("failure");
        assert_eq!(get_overall_ci_status(&[], &[], &[s]), "❌");
    }
}

pub fn print_multi_repo_summary(
    prs_by_repo: &[(String, Vec<PullRequest>)], 
    _username: &str, 
    org_name: &str
) {
    let total_prs: usize = prs_by_repo.iter().map(|(_, prs)| prs.len()).sum();
    let total_repos = prs_by_repo.len();
    
    if total_prs == 0 {
        return;
    }
    
    println!("{}", "=".repeat(60).blue());
    println!("📊 Your Current PR Status:");
    println!("You have {} open PRs across {} {} repositories:\n", total_prs, total_repos, org_name);
    
    let mut draft_count = 0;
    let mut ready_count = 0;
    
    for (_, prs) in prs_by_repo {
        for pr in prs {
            if pr.is_draft() {
                draft_count += 1;
            } else {
                ready_count += 1;
            }
        }
    }
    
    if ready_count > 0 {
        println!("🟢 Ready for Review:");
        for (repo_name, prs) in prs_by_repo {
            for pr in prs {
                if !pr.is_draft() {
                    println!("   • {} #{} - {}", repo_name, pr.number, truncate_string(&pr.title, 60));
                }
            }
        }
        println!();
    }
    
    if draft_count > 0 {
        println!("📝 Draft PRs:");
        for (repo_name, prs) in prs_by_repo {
            for pr in prs {
                if pr.is_draft() {
                    println!("   • {} #{} - {}", repo_name, pr.number, truncate_string(&pr.title, 60));
                }
            }
        }
        println!();
        
        let draft_text = if draft_count == 1 { "PR is" } else { "PRs are" };
        println!("📝 Note:");
        if draft_count == total_prs {
            println!("   All your {} in DRAFT status, so they're works in progress.", draft_text);
        } else {
            println!("   {} of your {} in DRAFT status, so they're works in progress.", draft_count, draft_text);
        }
    }
}

