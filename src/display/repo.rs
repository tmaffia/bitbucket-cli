use crate::api::models::Repository;
use crate::utils::formatting;
use comfy_table::{Attribute, Cell, Color};

pub fn print_repo_list(repos: &[Repository]) {
    if repos.is_empty() {
        crate::display::ui::info("No repositories found.");
        return;
    }

    let headers = vec!["Name", "Full Name", "Language", "Updated", "Private"];
    let rows: Vec<Vec<Cell>> = repos
        .iter()
        .map(|r| {
            vec![
                Cell::new(&r.name).add_attribute(Attribute::Bold),
                Cell::new(&r.full_name),
                Cell::new(r.language.as_deref().unwrap_or("-")),
                Cell::new(r.updated_on.as_deref().unwrap_or("-")),
                Cell::new(if r.is_private.unwrap_or(false) {
                    "Yes"
                } else {
                    "No"
                })
                .fg(if r.is_private.unwrap_or(false) {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ]
        })
        .collect();

    formatting::print_table(headers, rows);
}
