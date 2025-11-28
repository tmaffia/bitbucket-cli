use anyhow::Result;
use crossterm::style::{Color, Stylize};
use glob::Pattern;

use crate::display::ui::{display_in_pager, should_use_pager};

/// Display a diff with color formatting and optional paging
pub fn print_diff(
    diff_text: &str,
    patterns: &[String],
    max_diff_size: Option<usize>,
) -> Result<()> {
    let filtered_diff = filter_diff(diff_text, patterns, max_diff_size)?;
    let formatted = format_colored_diff(&filtered_diff);

    if should_use_pager() {
        display_in_pager(&formatted)?;
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

/// Display only the names of changed files from a diff
pub fn print_filenames_only(diff_text: &str, patterns: &[String]) {
    let compiled_patterns = compile_patterns(patterns);

    for line in diff_text.lines() {
        // Parse unified diff format: "diff --git a/path b/path"
        if line.starts_with("diff --git")
            && let Some(filename) = extract_filename_from_diff_line(line)
            && is_match(&filename, &compiled_patterns)
        {
            println!("{}", filename);
        }
    }
}

fn compile_patterns(patterns: &[String]) -> Vec<Pattern> {
    patterns
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect()
}

fn is_match(filename: &str, patterns: &[Pattern]) -> bool {
    if patterns.is_empty() {
        return true;
    }
    patterns.iter().any(|p| p.matches(filename))
}

fn filter_diff(
    diff_text: &str,
    patterns: &[String],
    max_diff_size: Option<usize>,
) -> Result<String> {
    if patterns.is_empty() && max_diff_size.is_none() {
        return Ok(diff_text.to_string());
    }

    let compiled_patterns = compile_patterns(patterns);
    let mut output = String::new();
    let mut current_file_diff = String::new();
    let mut current_filename: Option<String> = None;

    // Helper to process the accumulated chunk
    let mut process_chunk = |chunk: &str, filename: Option<&String>| {
        if let Some(fname) = filename {
            // Check pattern match
            if !is_match(fname, &compiled_patterns) {
                return;
            }

            // Check size limit
            if let Some(max_lines) = max_diff_size {
                let line_count = chunk.lines().count();
                if line_count > max_lines {
                    output.push_str(&format!("diff --git a/{} b/{}\n", fname, fname));
                    output.push_str(&format!(
                        "--- {} (skipped: diff too large, {} lines)\n",
                        fname, line_count
                    ));
                    output.push_str(&format!(
                        "+++ {} (skipped: diff too large, {} lines)\n",
                        fname, line_count
                    ));
                    return;
                }
            }
        }
        output.push_str(chunk);
    };

    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            // Process previous file
            if !current_file_diff.is_empty() {
                process_chunk(&current_file_diff, current_filename.as_ref());
                current_file_diff.clear();
            }

            // Start new file
            current_filename = extract_filename_from_diff_line(line);
        }
        current_file_diff.push_str(line);
        current_file_diff.push('\n');
    }

    // Process last file
    if !current_file_diff.is_empty() {
        process_chunk(&current_file_diff, current_filename.as_ref());
    }

    Ok(output)
}

/// Extract filename from a "diff --git a/path b/path" line
fn extract_filename_from_diff_line(line: &str) -> Option<String> {
    if let Some(rest) = line.strip_prefix("diff --git ")
        && let Some((_, dest)) = rest.split_once(" b/")
    {
        return Some(dest.to_string());
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

    #[test]
    fn test_filter_diff_pattern() {
        let diff = "diff --git a/file1.rs b/file1.rs\nindex 123..456 100644\n--- a/file1.rs\n+++ b/file1.rs\n@@ -1 +1 @@\n-old\n+new\ndiff --git a/file2.txt b/file2.txt\nindex 789..012 100644\n--- a/file2.txt\n+++ b/file2.txt\n@@ -1 +1 @@\n-foo\n+bar\n";
        let patterns = vec!["*.rs".to_string()];
        let filtered = filter_diff(diff, &patterns, None).unwrap();
        assert!(filtered.contains("file1.rs"));
        assert!(!filtered.contains("file2.txt"));
    }

    #[test]
    fn test_filter_diff_size() {
        let diff = "diff --git a/large.rs b/large.rs\nline1\nline2\nline3\nline4\nline5\n";
        let patterns = vec![];
        let filtered = filter_diff(diff, &patterns, Some(3)).unwrap();
        assert!(filtered.contains("skipped: diff too large"));
    }
}
