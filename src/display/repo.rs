use crate::api::models::Repository;
use crate::utils::formatting;
use comfy_table::{Attribute, Cell, Color};

pub fn print_repo_list(repos: &[Repository]) {
    if repos.is_empty() {
        crate::display::ui::info("No repositories found.");
        return;
    }

    let headers = vec!["Name", "Updated", "Visibility"];
    let rows: Vec<Vec<Cell>> = repos
        .iter()
        .map(|r| {
            let is_private = r.is_private.unwrap_or(false);
            vec![
                Cell::new(&r.name).add_attribute(Attribute::Bold),
                Cell::new(r.updated_on.as_deref().unwrap_or("-")),
                Cell::new(if is_private { "Private" } else { "Public" }).fg(if is_private {
                    Color::Yellow
                } else {
                    Color::Cyan
                }),
            ]
        })
        .collect();

    let table = formatting::format_table(headers, rows);

    if crate::display::ui::should_use_pager() {
        let content = format!("Found {} repositories:\n{}", repos.len(), table);
        if let Err(e) = crate::display::ui::display_in_pager(&content) {
            crate::display::ui::error(&format!("Failed to display in pager: {}", e));
        }
    } else {
        crate::display::ui::info(&format!("Found {} repositories:", repos.len()));
        println!("{}", table);
    }
}
