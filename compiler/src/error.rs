use colored::*;
use std::process::exit;

pub struct ErrorHandler;

impl ErrorHandler {
    pub fn error(
        empty: String,
        spacing: String,
        pointer: String,
        line_idx: usize,
        line_text: &str,
        error: &str,
    ) -> ! {
        println!("{} |", empty);
        println!("{} | {}", line_idx, line_text);
        println!("{} | {}{}", empty, spacing, pointer);
        println!("{}: {}", "Error".red(), error);

        exit(0);
    }

    pub fn error_tip(
        empty: String,
        spacing: String,
        pointer: String,
        line_idx: usize,
        line_text: &str,
        error: &str,
        tip: &str,
    ) -> ! {
        println!("{} |", empty);
        println!("{} | {}", line_idx, line_text);
        println!("{} | {}{}", empty, spacing, pointer);
        println!("{}: {}", "Error".red(), error);
        println!("{}: {}", "  Tip".blue(), tip);

        exit(0);
    }
}
