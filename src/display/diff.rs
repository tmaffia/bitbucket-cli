use anyhow::Result;
use crossterm::style::{Color, Stylize};

use crate::display::ui::{display_in_pager, should_use_pager};

/// Display a diff with color formatting and optional paging
pub fn print_diff(diff_text: &str) -> Result<()> {
    let formatted = format_colored_diff(diff_text);

    if should_use_pager() {
        display_in_pager(&formatted)?;
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

/// Display only the names of changed files from a diff
pub fn print_filenames_only(diff_text: &str) {
    for line in diff_text.lines() {
        // Parse unified diff format: "diff --git a/path b/path"
        if line.starts_with("diff --git") {
            if let Some(filename) = extract_filename_from_diff_line(line) {
                println!("{}", filename);
            }
        }
    }
}

/// Extract filename from a "diff --git a/path b/path" line
fn extract_filename_from_diff_line(line: &str) -> Option<String> {
    // Format: "diff --git a/filename b/filename"
    // We want the "b/" version (destination file)
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 4 && parts[0] == "diff" && parts[1] == "--git" {
        // parts[3] is "b/filename"
        let path = parts[3].strip_prefix("b/").unwrap_or(parts[3]);
        return Some(path.to_string());
    }
    None
}

/// Format a diff with colors
fn format_colored_diff(diff_text: &str) -> String {
    let mut output = String::new();

    for line in diff_text.lines() {
        let colored_line = if line.starts_with("+++") || line.starts_with("---") {
            // File headers - bold white
            format!("{}\n", line.bold())
        } else if line.starts_with("@@") {
            // Hunk headers - cyan
            format!("{}\n", line.with(Color::Cyan))
        } else if line.starts_with('+') {
            // Additions - green
            format!("{}\n", line.with(Color::Green))
        } else if line.starts_with('-') {
            // Deletions - red
            format!("{}\n", line.with(Color::Red))
        } else if line.starts_with("diff --git") || line.starts_with("index ") {
            // Diff metadata - bold
            format!("{}\n", line.bold())
        } else {
            // Context lines - dark grey
            format!("{}\n", line.with(Color::DarkGrey))
        };

        output.push_str(&colored_line);
    }

    output
}
