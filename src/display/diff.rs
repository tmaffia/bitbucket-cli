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
    if let Some(rest) = line.strip_prefix("diff --git ") {
        if let Some((_, dest)) = rest.split_once(" b/") {
            return Some(dest.to_string());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename_valid() {
        let line = "diff --git a/src/main.rs b/src/main.rs";
        let filename = extract_filename_from_diff_line(line);
        assert_eq!(filename, Some("src/main.rs".to_string()));
    }

    #[test]
    fn test_extract_filename_invalid_prefix() {
        let line = "something else";
        let filename = extract_filename_from_diff_line(line);
        assert_eq!(filename, None);
    }

    #[test]
    fn test_extract_filename_invalid_format() {
        let line = "diff --git just_one_path";
        let filename = extract_filename_from_diff_line(line);
        assert_eq!(filename, None);
    }
}
