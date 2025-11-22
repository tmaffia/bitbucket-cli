use crossterm::style::{Color, Stylize};
use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Enable or disable verbose logging
pub fn set_enabled(enabled: bool) {
    VERBOSE.store(enabled, Ordering::Relaxed);
}

/// Check if verbose logging is enabled
pub fn is_enabled() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Log a debug message if verbose mode is enabled
pub fn log(message: &str) {
    if is_enabled() {
        eprintln!("{} {}", "DEBUG:".with(Color::Magenta).bold(), message);
    }
}
