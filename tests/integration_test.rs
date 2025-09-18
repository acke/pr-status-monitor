use my_prs::github::PullRequest;
use my_prs::output::print_pr_summary;

#[test]
fn test_pr_summary_output() {
    // Create a mock PR for testing
    let pr = PullRequest {
        number: 123,
        title: "Test PR".to_string(),
        state: "open".to_string(),
        user: my_prs::github::User {
            login: "testuser".to_string(),
            id: 1,
            avatar_url: "https://example.com/avatar.png".to_string(),
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        html_url: "https://github.com/owner/repo/pull/123".to_string(),
        head: my_prs::github::Branch {
            label: "feature-branch".to_string(),
            r#ref: "feature-branch".to_string(),
            sha: "abc123".to_string(),
        },
        base: my_prs::github::Branch {
            label: "main".to_string(),
            r#ref: "main".to_string(),
            sha: "def456".to_string(),
        },
        draft: false,
        mergeable: Some(true),
        mergeable_state: Some("clean".to_string()),
    };

    // This test just ensures the function doesn't panic
    print_pr_summary(&pr);
}
