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
    if !comments.is_empty() {
        println!("\nComments:");
        for comment in comments {
            println!("--------------------------------------------------");
            println!(
                "Author: {} ({})",
                comment.user.display_name, comment.created_on
            );
            if let Some(inline) = &comment.inline {
                println!("File: {}", inline.path);
                if let Some(line) = inline.to.or(inline.from) {
                    println!("Line: {}", line);
                }
            }
            println!("\n{}", comment.content.raw);
        }
    }
}
