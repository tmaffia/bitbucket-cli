/// Display utilities for user-facing output

/// Print a success message with green checkmark
pub fn success(message: &str) {
    println!("✅ {}", message);
}

/// Print an error message with red X
pub fn error(message: &str) {
    eprintln!("❌ {}", message);
}

/// Print a warning message with yellow triangle
pub fn warning(message: &str) {
    println!("⚠️  {}", message);
}

/// Print an info message
pub fn info(message: &str) {
    println!("ℹ️  {}", message);
}
