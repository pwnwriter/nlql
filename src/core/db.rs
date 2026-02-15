// database connection and query execution
// supports postgres, sqlite, and mysql

use crate::Error;
use serde::Serialize;
use sqlx::{any::AnyPoolOptions, AnyPool, Column, Row};

pub struct Db {
    pool: AnyPool,
    dialect: Dialect,
}

#[derive(Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

enum Dialect {
    Postgres,
    Sqlite,
    Mysql,
}

impl Db {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        sqlx::any::install_default_drivers();

        // figure out which database we're talking to
        let dialect = detect_dialect(url);

        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;

        Ok(Self { pool, dialect })
    }

    // get table and column info so claude knows what to query
    pub async fn schema(&self) -> Result<String, Error> {
        match self.dialect {
            Dialect::Postgres => self.postgres_schema().await,
            Dialect::Sqlite => self.sqlite_schema().await,
            Dialect::Mysql => self.mysql_schema().await,
        }
    }

    async fn postgres_schema(&self) -> Result<String, Error> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"SELECT table_name::text, column_name::text, data_type::text
               FROM information_schema.columns
               WHERE table_schema = 'public'
               ORDER BY table_name, ordinal_position"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(format_schema(rows))
    }

    async fn sqlite_schema(&self) -> Result<String, Error> {
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for (table,) in tables {
            let query = format!("PRAGMA table_info(\"{}\")", table);
            let cols: Vec<(i32, String, String, i32, Option<String>, i32)> =
                sqlx::query_as(&query).fetch_all(&self.pool).await?;

            for (_, name, dtype, _, _, _) in cols {
                result.push((table.clone(), name, dtype));
            }
        }

        Ok(format_schema(result))
    }

    async fn mysql_schema(&self) -> Result<String, Error> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"SELECT table_name, column_name, data_type
               FROM information_schema.columns
               WHERE table_schema = DATABASE()
               ORDER BY table_name, ordinal_position"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(format_schema(rows))
    }

    // run the sql and return results as json
    pub async fn execute(&self, sql: &str) -> Result<QueryResult, Error> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
            });
        }

        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        let json_rows: Vec<Vec<serde_json::Value>> = rows
            .iter()
            .map(|row| {
                columns
                    .iter()
                    .enumerate()
                    .map(|(i, _)| row_value_to_json(row, i))
                    .collect()
            })
            .collect();

        let row_count = json_rows.len();

        Ok(QueryResult {
            columns,
            rows: json_rows,
            row_count,
        })
    }

    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }
}

// figure out dialect from connection string
fn detect_dialect(url: &str) -> Dialect {
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Dialect::Postgres
    } else if url.starts_with("mysql://") || url.starts_with("mariadb://") {
        Dialect::Mysql
    } else {
        Dialect::Sqlite
    }
}

// turn schema rows into readable text for claude
fn format_schema(rows: Vec<(String, String, String)>) -> String {
    let mut result = String::new();
    let mut current_table = String::new();

    for (table, column, dtype) in rows {
        if table != current_table {
            if !current_table.is_empty() {
                result.push_str(")\n\n");
            }
            result.push_str(&format!("TABLE {table} (\n"));
            current_table = table;
        }
        result.push_str(&format!("  {column} {dtype}\n"));
    }

    if !current_table.is_empty() {
        result.push(')');
    }

    result
}

// convert database values to json (handling type mismatches gracefully)
fn row_value_to_json(row: &sqlx::any::AnyRow, index: usize) -> serde_json::Value {
    use sqlx::ValueRef;

    // null check first
    if row.try_get_raw(index).map(|v| v.is_null()).unwrap_or(true) {
        return serde_json::Value::Null;
    }

    // try types in order of how common they are
    if let Ok(v) = row.try_get::<String, _>(index) {
        return serde_json::Value::String(v);
    }
    if let Ok(v) = row.try_get::<i64, _>(index) {
        return serde_json::Value::Number(v.into());
    }
    if let Ok(v) = row.try_get::<i32, _>(index) {
        return serde_json::Value::Number(v.into());
    }
    if let Ok(v) = row.try_get::<f64, _>(index) {
        return serde_json::Number::from_f64(v)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null);
    }
    if let Ok(v) = row.try_get::<bool, _>(index) {
        return serde_json::Value::Bool(v);
    }

    // give up - some postgres types just don't work with the any driver
    serde_json::Value::String("<unsupported>".to_string())
}
