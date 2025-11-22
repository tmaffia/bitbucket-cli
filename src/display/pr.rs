use crate::api::models::{Comment, CommitStatus, PullRequest};
use crate::utils::formatting;
use comfy_table::{Attribute, Cell, Color};

pub fn print_pr_details(pr: &PullRequest, statuses: &[CommitStatus]) {
    // Display PR details
    let mut details = vec![
        ("ID", pr.id.to_string()),
        ("Title", pr.title.clone()),
        ("Author", pr.author.display_name.clone()),
        ("State", pr.state.clone()),
        ("Source", pr.source.branch.name.clone()),
        ("Destination", pr.destination.branch.name.clone()),
        ("Link", pr.links.html.href.clone()),
    ];

    if let Some(desc) = &pr.description {
        details.push(("Description", desc.clone()));
    }

    formatting::print_key_value_table(
        details
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect::<Vec<_>>(),
    );

    // Display Approvals
    let approvals: Vec<&crate::api::models::Participant> =
        pr.participants.iter().filter(|p| p.approved).collect();

    if !approvals.is_empty() {
        println!("\nApprovals:");
        for p in approvals {
            println!("- {}", p.user.display_name);
        }
    }

    // Display Build Status
    if !statuses.is_empty() {
        println!("\nBuild Status:");
        let headers = vec!["Pipeline", "Status", "URL"];
        let rows = statuses
            .iter()
            .map(|status| {
                let (status_text, color) = match status.state.as_str() {
                    "SUCCESSFUL" => ("SUCCESSFUL", Color::Green),
                    "FAILED" => ("FAILED", Color::Red),
                    "INPROGRESS" => ("INPROGRESS", Color::Yellow),
                    "STOPPED" => ("STOPPED", Color::Grey),
                    _ => (status.state.as_str(), Color::White),
                };
                vec![
                    Cell::new(status.name.clone().unwrap_or_else(|| status.key.clone())),
                    Cell::new(status_text)
                        .fg(color)
                        .add_attribute(Attribute::Bold),
                    Cell::new(status.url.clone()),
                ]
            })
            .collect();
        formatting::print_table(headers, rows);
    }
}

pub fn print_comments(comments: &[Comment]) {
    if comments.is_empty() {
        return;
    }

    println!("\nComments:");
    for (idx, comment) in comments.iter().enumerate() {
        if idx > 0 {
            println!(); // Add spacing between comments
        }

        let mut details = vec![
            ("Author", comment.user.display_name.clone()),
            ("Created", comment.created_on.clone()),
        ];

        // Add inline context if present
        if let Some(inline) = &comment.inline {
            details.push(("File", inline.path.clone()));
            if let Some(line) = inline.to.or(inline.from) {
                details.push(("Line", line.to_string()));
            }
        }

        details.push(("Comment", comment.content.raw.clone()));

        formatting::print_key_value_table(
            details
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<Vec<_>>(),
        );
    }
}

pub fn print_pr_list(prs: &[PullRequest]) {
    let output = format_pr_list(prs);
    println!("{}", output);
}

pub fn format_pr_list(prs: &[PullRequest]) -> String {
    let headers = vec!["ID", "Title", "Author", "Source", "State", "Updated"];
    let rows: Vec<Vec<Cell>> = prs
        .iter()
        .map(|pr| {
            vec![
                Cell::new(pr.id.to_string()),
                Cell::new(&pr.title),
                Cell::new(&pr.author.display_name),
                Cell::new(&pr.source.branch.name),
                Cell::new(&pr.state),
                Cell::new(&pr.updated_on),
            ]
        })
        .collect();

    formatting::format_table(headers, rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::*;

    fn create_mock_pr(id: u32, title: &str) -> PullRequest {
        PullRequest {
            id,
            title: title.to_string(),
            description: None,
            state: "OPEN".to_string(),
            created_on: "2023-01-01".to_string(),
            updated_on: "2023-01-02".to_string(),
            author: User {
                display_name: "Author Name".to_string(),
                uuid: "123".to_string(),
                nickname: None,
            },
            source: Source {
                branch: Branch {
                    name: "feature/branch".to_string(),
                },
                repository: Repository {
                    name: "repo".to_string(),
                    full_name: "owner/repo".to_string(),
                    uuid: "456".to_string(),
                },
                commit: None,
            },
            destination: Source {
                branch: Branch {
                    name: "main".to_string(),
                },
                repository: Repository {
                    name: "repo".to_string(),
                    full_name: "owner/repo".to_string(),
                    uuid: "456".to_string(),
                },
                commit: None,
            },
            links: Links {
                html: Link {
                    href: "http://example.com".to_string(),
                },
            },
            participants: vec![],
        }
    }

    #[test]
    fn test_format_pr_list() {
        let prs = vec![
            create_mock_pr(1, "PR Title 1"),
            create_mock_pr(2, "PR Title 2"),
        ];

        let output = format_pr_list(&prs);
        let lines: Vec<&str> = output.lines().collect();

        // Find header
        let header_idx = lines.iter().position(|l| l.contains("ID") && l.contains("Title") && l.contains("Author"));
        assert!(header_idx.is_some(), "Header not found");

        // Find PR 1
        let pr1_idx = lines.iter().position(|l| l.contains("1") && l.contains("PR Title 1") && l.contains("Author Name"));
        assert!(pr1_idx.is_some(), "PR 1 row not found");

        // Find PR 2
        let pr2_idx = lines.iter().position(|l| l.contains("2") && l.contains("PR Title 2"));
        assert!(pr2_idx.is_some(), "PR 2 row not found");

        // Verify Order
        assert!(header_idx.unwrap() < pr1_idx.unwrap());
        assert!(pr1_idx.unwrap() < pr2_idx.unwrap());
    }

    #[test]
    fn test_format_pr_list_empty() {
        let prs: Vec<PullRequest> = vec![];
        let output = format_pr_list(&prs);
        assert!(output.contains("ID"));
        assert!(output.contains("Title"));
    }
}
