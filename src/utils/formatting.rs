use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

/// Apply consistent styling to all tables
fn apply_table_style(table: &mut Table) {
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
}

pub fn print_key_value_table(data: Vec<(&str, String)>) {
    let mut table = Table::new();
    apply_table_style(&mut table);
    table.set_width(80).set_header(vec!["Key", "Value"]);

    for (key, value) in data {
        table.add_row(vec![
            Cell::new(key).add_attribute(Attribute::Bold),
            Cell::new(value),
        ]);
    }

    println!("{}", table);
}

pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<Cell>>) {
    let table = format_table(headers, rows);
    println!("{}", table);
}

pub fn format_table(headers: Vec<&str>, rows: Vec<Vec<Cell>>) -> String {
    let mut table = Table::new();
    apply_table_style(&mut table);
    table.set_header(headers);

    for row in rows {
        table.add_row(row);
    }

    table.to_string()
}
