// output formatting - pretty tables or raw json

use crate::core::QueryResult;

pub struct Output;

impl Output {
    // nice table format for humans
    pub fn pretty(sql: &str, result: &QueryResult) {
        println!("sql: {sql}\n");
        println!("rows: {}\n", result.row_count);

        if result.rows.is_empty() {
            println!("no results");
            return;
        }

        // figure out column widths
        let mut widths: Vec<usize> = result.columns.iter().map(|c| c.len()).collect();

        for row in &result.rows {
            for (i, val) in row.iter().enumerate() {
                let len = format_value(val).len();
                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }

        // cap at 40 so things don't get crazy
        for w in &mut widths {
            if *w > 40 {
                *w = 40;
            }
        }

        // header
        let header: Vec<String> = result
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:width$}", c, width = widths[i]))
            .collect();
        println!("{}", header.join(" | "));

        // separator
        let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
        println!("{}", sep.join("-+-"));

        // rows
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

    // raw json for scripts
    pub fn raw(result: &QueryResult) {
        println!("{}", serde_json::to_string(result).unwrap_or_default());
    }
}

fn format_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => val.to_string(),
    }
}
