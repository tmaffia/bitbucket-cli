use anyhow::Result;
use serde::Serialize;
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};

/// Display utilities for user-facing output
use crossterm::style::{Color, Stylize};

/// Print a success message with green text
pub fn success(message: &str) {
    println!("{} {}", "SUCCESS:".with(Color::Green).bold(), message);
}

/// Print an error message with red text
pub fn error(message: &str) {
    eprintln!("{}   {}", "ERROR:".with(Color::Red).bold(), message);
}

/// Print a warning message with yellow text
pub fn warning(message: &str) {
    println!("{} {}", "WARNING:".with(Color::Yellow).bold(), message);
}

/// Print an info message
pub fn info(message: &str) {
    println!("{}    {}", "INFO:".with(Color::Blue).bold(), message);
}

/// Print data as JSON
pub fn print_json<T: Serialize>(data: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

/// Check if we should use a pager (only if output is to a TTY)
pub fn should_use_pager() -> bool {
    // Check if stdout is a terminal
    std::io::stdout().is_terminal()
}

/// Display content in a pager (less -R by default)
pub fn display_in_pager(content: &str) -> Result<()> {
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
