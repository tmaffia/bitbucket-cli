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

#[cfg(test)]
mod tests {
    use super::*;
    use comfy_table::Cell;

    #[test]
    fn test_format_table() {
        let headers = vec!["Name", "Age"];
        let rows = vec![
            vec![Cell::new("Alice"), Cell::new("30")],
            vec![Cell::new("Bob"), Cell::new("25")],
        ];

        let output = format_table(headers, rows);
        let lines: Vec<&str> = output.lines().collect();

        // Simple structural check logic:
        // 1. Find header line
        // 2. Find Alice's line
        // 3. Find Bob's line
        // 4. Ensure order matches

        let header_line_idx = lines.iter().position(|l| l.contains("Name") && l.contains("Age"));
        assert!(header_line_idx.is_some(), "Header line not found");

        let alice_line_idx = lines.iter().position(|l| l.contains("Alice") && l.contains("30"));
        assert!(alice_line_idx.is_some(), "Row for Alice not found");

        let bob_line_idx = lines.iter().position(|l| l.contains("Bob") && l.contains("25"));
        assert!(bob_line_idx.is_some(), "Row for Bob not found");

        // Verify order: Header -> Alice -> Bob
        let h_idx = header_line_idx.unwrap();
        let a_idx = alice_line_idx.unwrap();
        let b_idx = bob_line_idx.unwrap();

        assert!(h_idx < a_idx, "Header should appear before Alice");
        assert!(a_idx < b_idx, "Alice should appear before Bob");
    }

    #[test]
    fn test_format_table_empty() {
        let headers = vec!["Col1", "Col2"];
        let rows: Vec<Vec<Cell>> = vec![];

        let output = format_table(headers, rows);

        assert!(output.contains("Col1"));
        assert!(output.contains("Col2"));
    }
}
