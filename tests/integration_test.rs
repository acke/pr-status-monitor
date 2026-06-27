use my_prs::github::{Branch, CheckRun, PullRequest, StatusCheck, User, WorkflowRun};
use my_prs::output::{get_overall_ci_status, print_pr_summary};

fn make_pr(number: u64, state: &str, draft: bool) -> PullRequest {
    PullRequest {
        number,
        title: format!("PR #{}", number),
        state: state.to_string(),
        user: User { login: "testuser".to_string(), id: 1, avatar_url: String::new() },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        html_url: format!("https://github.com/owner/repo/pull/{}", number),
        head: Branch { label: "feature".to_string(), r#ref: "feature".to_string(), sha: "abc".to_string() },
        base: Branch { label: "main".to_string(), r#ref: "main".to_string(), sha: "def".to_string() },
        draft,
        mergeable: Some(true),
        mergeable_state: Some("clean".to_string()),
    }
}

fn make_workflow(name: &str, status: &str, conclusion: Option<&str>) -> WorkflowRun {
    WorkflowRun {
        id: 1,
        name: name.to_string(),
        status: status.to_string(),
        conclusion: conclusion.map(str::to_string),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        html_url: String::new(),
        workflow_id: 1,
    }
}

#[test]
fn test_pr_summary_does_not_panic() {
    let pr = make_pr(123, "open", false);
    print_pr_summary(&pr);
}

#[test]
fn test_pr_summary_draft_does_not_panic() {
    let pr = make_pr(42, "open", true);
    print_pr_summary(&pr);
}

#[test]
fn test_pr_summary_closed_does_not_panic() {
    let pr = make_pr(99, "closed", false);
    print_pr_summary(&pr);
}

#[test]
fn test_ci_status_success_end_to_end() {
    let wf = make_workflow("Tests", "completed", Some("success"));
    assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "✅");
}

#[test]
fn test_ci_status_in_progress_end_to_end() {
    let wf = make_workflow("Tests", "in_progress", None);
    assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "🔄");
}

#[test]
fn test_ci_status_failure_end_to_end() {
    let wf = make_workflow("Tests", "completed", Some("failure"));
    assert_eq!(get_overall_ci_status(&[wf], &[], &[]), "❌");
}

#[test]
fn test_ci_status_mixed_running_and_failed() {
    let failing = make_workflow("Tests", "completed", Some("failure"));
    let running = WorkflowRun {
        id: 2,
        name: "Lint".to_string(),
        status: "in_progress".to_string(),
        conclusion: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        html_url: String::new(),
        workflow_id: 2,
    };
    // Running takes priority over failure
    assert_eq!(get_overall_ci_status(&[failing, running], &[], &[]), "🔄");
}

#[test]
fn test_ci_status_with_check_runs() {
    let check = CheckRun {
        id: 1,
        name: "unit-tests".to_string(),
        status: "completed".to_string(),
        conclusion: Some("success".to_string()),
        started_at: None,
        completed_at: None,
        html_url: String::new(),
    };
    assert_eq!(get_overall_ci_status(&[], &[check], &[]), "✅");
}

#[test]
fn test_ci_status_with_status_checks() {
    let s = StatusCheck {
        state: "success".to_string(),
        description: Some("All good".to_string()),
        context: "ci/test".to_string(),
        target_url: None,
    };
    assert_eq!(get_overall_ci_status(&[], &[], &[s]), "✅");
}
