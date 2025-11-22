use anyhow::Result;
use serde::Serialize;

/// Display utilities for user-facing output
use crossterm::style::{Color, Stylize};

/// Display utilities for user-facing output

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
