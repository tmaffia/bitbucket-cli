use anyhow::Result;
use crossterm::style::{Color, Stylize};
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};

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

/// Check if we should use a pager (only if output is to a TTY)
fn should_use_pager() -> bool {
    // Check if stdout is a terminal
    std::io::stdout().is_terminal()
}

/// Display content in a pager (less -R by default)
fn display_in_pager(content: &str) -> Result<()> {
    // Try to use $PAGER environment variable, fallback to less -R
    let pager_cmd = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());

    // Split command and args
    let mut parts = pager_cmd.split_whitespace();
    let cmd = parts.next().unwrap_or("less");
    let args: Vec<&str> = parts.collect();

    // Start the pager process
    let child = Command::new(cmd)
        .args(&args)
        .arg("-R") // Enable color codes
        .stdin(Stdio::piped())
        .spawn();

    match child {
        Ok(mut process) => {
            if let Some(mut stdin) = process.stdin.take() {
                // Write content to pager stdin, ignore broken pipe errors
                // (happens when user quits with 'q')
                let _ = stdin.write_all(content.as_bytes());
                drop(stdin); // Close stdin to signal EOF
            }

            // Wait for pager to finish, ignore any exit errors
            let _ = process.wait();
            Ok(())
        }
        Err(_) => {
            // If pager fails, just print directly
            print!("{}", content);
            Ok(())
        }
    }
}
