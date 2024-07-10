use jira_api::{Issue, JiraClient};

pub async fn pull(client: &JiraClient, jql: &str) -> Vec<Issue> {
    let mut start_at = 0;
    let mut issues = vec![];
    let mut total = 500;

    while start_at < total {
        let mut response = client
            .search_issues(jql, start_at, 500, Some(&["status"]), None)
            .await
            .unwrap();

        total = response.total;
        start_at += response.max_results;

        issues.append(&mut response.issues);

        if total <= issues.len() as u32 {
            break;
        }
    }

    issues
}
