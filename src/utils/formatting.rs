use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub fn print_key_value_table(data: Vec<(&str, String)>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(80)
        .set_header(vec!["Key", "Value"]);

    for (key, value) in data {
        table.add_row(vec![
            Cell::new(key).add_attribute(Attribute::Bold),
            Cell::new(value),
        ]);
    }

    println!("{}", table);
}

pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<Cell>>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);

    for row in rows {
        table.add_row(row);
    }

    println!("{}", table);
}
