use crate::db::QueryResult;

pub struct Output;

impl Output {
    pub fn pretty(sql: &str, result: &QueryResult) {
        println!("SQL: {sql}\n");
        println!("Rows: {}\n", result.row_count);

        if result.rows.is_empty() {
            println!("No results.");
            return;
        }

        // Calculate column widths
        let mut widths: Vec<usize> = result.columns.iter().map(|c| c.len()).collect();

        for row in &result.rows {
            for (i, val) in row.iter().enumerate() {
                let len = format_value(val).len();
                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }

        // Cap widths at 40 chars
        for w in &mut widths {
            if *w > 40 {
                *w = 40;
            }
        }

        // Print header
        let header: Vec<String> = result
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:width$}", c, width = widths[i]))
            .collect();
        println!("{}", header.join(" | "));

        // Print separator
        let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
        println!("{}", sep.join("-+-"));

        // Print rows
        for row in &result.rows {
            let formatted: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let s = format_value(v);
                    let s = if s.len() > 40 {
                        format!("{}...", &s[..37])
                    } else {
                        s
                    };
                    format!("{:width$}", s, width = widths[i])
                })
                .collect();
            println!("{}", formatted.join(" | "));
        }
    }

    pub fn raw(result: &QueryResult) {
        println!("{}", serde_json::to_string(result).unwrap_or_default());
    }
}

fn format_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => val.to_string(),
    }
}
