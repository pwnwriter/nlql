// app state for the tui

use crate::core::QueryResult;
use crate::tui::theme::{Theme, ThemeKind};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Prompt,
    Sql,
    Results,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popup {
    None,
    Themes,
    Confirm,
    Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Ok,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,     // SELECT queries
    Moderate, // INSERT, UPDATE with WHERE
    Danger,   // DELETE, DROP, TRUNCATE, UPDATE without WHERE
}

impl RiskLevel {
    pub fn from_sql(sql: &str) -> Self {
        let upper = sql.to_uppercase();
        let trimmed = upper.trim();

        if trimmed.starts_with("DROP")
            || trimmed.starts_with("TRUNCATE")
            || trimmed.starts_with("ALTER")
        {
            return RiskLevel::Danger;
        }

        if trimmed.starts_with("DELETE") {
            if upper.contains("WHERE") {
                return RiskLevel::Moderate;
            }
            return RiskLevel::Danger;
        }

        if trimmed.starts_with("UPDATE") {
            if upper.contains("WHERE") {
                return RiskLevel::Moderate;
            }
            return RiskLevel::Danger;
        }

        if trimmed.starts_with("INSERT") {
            return RiskLevel::Moderate;
        }

        RiskLevel::Safe
    }

    pub fn label(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "SAFE",
            RiskLevel::Moderate => "MODERATE",
            RiskLevel::Danger => "DANGER",
        }
    }

    pub fn sql_type(&self, sql: &str) -> &'static str {
        let upper = sql.to_uppercase();
        let trimmed = upper.trim();

        if trimmed.starts_with("SELECT") {
            "SELECT"
        } else if trimmed.starts_with("INSERT") {
            "INSERT"
        } else if trimmed.starts_with("UPDATE") {
            "UPDATE"
        } else if trimmed.starts_with("DELETE") {
            "DELETE"
        } else if trimmed.starts_with("DROP") {
            "DROP"
        } else if trimmed.starts_with("TRUNCATE") {
            "TRUNCATE"
        } else if trimmed.starts_with("ALTER") {
            "ALTER"
        } else if trimmed.starts_with("CREATE") {
            "CREATE"
        } else {
            "QUERY"
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DbInfo {
    pub dialect: String,
    pub host: String,
    pub database: String,
    pub tables: usize,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub model: String,
}

pub struct App {
    pub running: bool,
    pub mode: Mode,
    pub panel: Panel,
    pub popup: Popup,
    pub theme_kind: ThemeKind,
    pub theme: Theme,

    // settings
    pub confirm_before_run: bool,
    pub cache_enabled: bool,

    // database info
    pub db_info: DbInfo,
    pub agent_info: AgentInfo,

    // prompt input (multi-line)
    pub prompt: String,
    pub prompt_cursor: usize,

    // connection editor
    pub connection_input: String,
    pub connection_cursor: usize,

    // sql state
    pub sql: Option<String>,
    pub sql_status: Option<String>,
    pub latency_ms: Option<u64>,
    pub confidence: Option<u8>,
    pub risk: Option<RiskLevel>,
    pub show_explain: bool,
    pub explain_result: Option<String>,

    // results
    pub result: Option<QueryResult>,
    pub error: Option<String>,

    // logs
    pub logs: Vec<LogEntry>,

    // state
    pub loading: bool,
    pub reconnecting: bool,
    pub query_start: Option<Instant>,

    // scroll
    pub result_scroll: usize,
    pub log_scroll: usize,
    pub theme_scroll: usize,

    // history
    pub history: Vec<String>,
    pub history_index: Option<usize>,
}

impl App {
    pub fn new(schema: String, db_info: DbInfo, confirm_before_run: bool) -> Self {
        let theme_kind = ThemeKind::CatppuccinMocha;
        let connection_input = db_info.url.clone();

        let mut app = Self {
            running: true,
            mode: Mode::Normal,
            panel: Panel::Prompt,
            popup: Popup::None,
            theme_kind,
            theme: Theme::from_kind(theme_kind),
            confirm_before_run,
            cache_enabled: false,
            db_info: db_info.clone(),
            agent_info: AgentInfo {
                name: "nlql-agent".to_string(),
                model: "claude-sonnet-4".to_string(),
            },
            prompt: String::new(),
            prompt_cursor: 0,
            connection_input,
            connection_cursor: 0,
            sql: None,
            sql_status: None,
            latency_ms: None,
            confidence: None,
            risk: None,
            show_explain: false,
            explain_result: None,
            result: None,
            error: None,
            logs: Vec::new(),
            loading: false,
            reconnecting: false,
            query_start: None,
            result_scroll: 0,
            log_scroll: 0,
            theme_scroll: theme_kind.index(),
            history: Vec::new(),
            history_index: None,
        };

        // initial log
        app.log(LogLevel::Ok, format!("connected {}", db_info.dialect));
        app.log(
            LogLevel::Ok,
            format!("agent selected: {}", app.agent_info.name),
        );
        app.log(
            LogLevel::Info,
            format!(
                "schema loaded ({} tables)",
                schema.matches("TABLE ").count()
            ),
        );

        app
    }

    pub fn log(&mut self, level: LogLevel, message: String) {
        self.logs.push(LogEntry { level, message });
        // auto-scroll to bottom
        if self.logs.len() > 1 {
            self.log_scroll = self.logs.len().saturating_sub(10);
        }
    }

    pub fn set_theme(&mut self, kind: ThemeKind) {
        self.theme_kind = kind;
        self.theme = Theme::from_kind(kind);
        self.theme_scroll = kind.index();
    }

    pub fn open_theme_popup(&mut self) {
        self.popup = Popup::Themes;
        self.theme_scroll = self.theme_kind.index();
    }

    pub fn open_connection_popup(&mut self) {
        self.popup = Popup::Connection;
        self.connection_input = self.db_info.url.clone();
        self.connection_cursor = self.connection_input.len();
    }

    pub fn close_popup(&mut self) {
        self.popup = Popup::None;
    }

    pub fn theme_scroll_up(&mut self) {
        if self.theme_scroll > 0 {
            self.theme_scroll -= 1;
            self.set_theme(ThemeKind::ALL[self.theme_scroll]);
        }
    }

    pub fn theme_scroll_down(&mut self) {
        if self.theme_scroll < ThemeKind::ALL.len() - 1 {
            self.theme_scroll += 1;
            self.set_theme(ThemeKind::ALL[self.theme_scroll]);
        }
    }

    pub fn select_theme(&mut self) {
        self.set_theme(ThemeKind::ALL[self.theme_scroll]);
        self.close_popup();
    }

    pub fn show_confirm(&mut self, sql: String) {
        self.sql = Some(sql);
        self.popup = Popup::Confirm;
    }

    pub fn confirm_sql(&mut self) -> Option<String> {
        self.popup = Popup::None;
        self.sql.clone()
    }

    pub fn cancel_sql(&mut self) {
        self.popup = Popup::None;
        self.sql = None;
        self.sql_status = None;
    }

    // connection input editing
    pub fn connection_insert_char(&mut self, c: char) {
        self.connection_input.insert(self.connection_cursor, c);
        self.connection_cursor += 1;
    }

    pub fn connection_delete_char(&mut self) {
        if self.connection_cursor > 0 {
            self.connection_cursor -= 1;
            self.connection_input.remove(self.connection_cursor);
        }
    }

    pub fn connection_delete_char_forward(&mut self) {
        if self.connection_cursor < self.connection_input.len() {
            self.connection_input.remove(self.connection_cursor);
        }
    }

    pub fn connection_move_left(&mut self) {
        self.connection_cursor = self.connection_cursor.saturating_sub(1);
    }

    pub fn connection_move_right(&mut self) {
        if self.connection_cursor < self.connection_input.len() {
            self.connection_cursor += 1;
        }
    }

    pub fn connection_move_start(&mut self) {
        self.connection_cursor = 0;
    }

    pub fn connection_move_end(&mut self) {
        self.connection_cursor = self.connection_input.len();
    }

    pub fn connection_clear(&mut self) {
        self.connection_input.clear();
        self.connection_cursor = 0;
    }

    pub fn submit_connection(&mut self) -> Option<String> {
        if self.connection_input.is_empty() {
            return None;
        }
        let url = self.connection_input.clone();
        self.popup = Popup::None;
        Some(url)
    }

    pub fn update_db_info(&mut self, info: DbInfo, schema: String) {
        self.log(LogLevel::Ok, format!("connected {}", info.dialect));
        self.log(
            LogLevel::Info,
            format!(
                "schema loaded ({} tables)",
                schema.matches("TABLE ").count()
            ),
        );
        self.db_info = info;
        self.reconnecting = false;
        self.result = None;
        self.sql = None;
        self.sql_status = None;
        self.error = None;
        self.confidence = None;
        self.risk = None;
        self.show_explain = false;
        self.explain_result = None;
    }

    pub fn cycle_panel(&mut self) {
        self.panel = match self.panel {
            Panel::Prompt => Panel::Sql,
            Panel::Sql => Panel::Results,
            Panel::Results => Panel::Logs,
            Panel::Logs => Panel::Prompt,
        };
    }

    pub fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn exit_insert(&mut self) {
        self.mode = Mode::Normal;
    }

    // prompt editing
    pub fn insert_char(&mut self, c: char) {
        self.prompt.insert(self.prompt_cursor, c);
        self.prompt_cursor += 1;
    }

    pub fn insert_newline(&mut self) {
        self.prompt.insert(self.prompt_cursor, '\n');
        self.prompt_cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.prompt_cursor > 0 {
            self.prompt_cursor -= 1;
            self.prompt.remove(self.prompt_cursor);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.prompt_cursor < self.prompt.len() {
            self.prompt.remove(self.prompt_cursor);
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.prompt_cursor = self.prompt_cursor.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        if self.prompt_cursor < self.prompt.len() {
            self.prompt_cursor += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.prompt_cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.prompt_cursor = self.prompt.len();
    }

    pub fn clear_prompt(&mut self) {
        self.prompt.clear();
        self.prompt_cursor = 0;
    }

    // history navigation
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
            }
            Some(i) if i > 0 => {
                self.history_index = Some(i - 1);
            }
            _ => {}
        }
        if let Some(i) = self.history_index {
            self.prompt = self.history[i].clone();
            self.prompt_cursor = self.prompt.len();
        }
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            Some(i) if i < self.history.len() - 1 => {
                self.history_index = Some(i + 1);
                self.prompt = self.history[i + 1].clone();
                self.prompt_cursor = self.prompt.len();
            }
            Some(_) => {
                self.history_index = None;
                self.clear_prompt();
            }
            None => {}
        }
    }

    pub fn submit(&mut self) -> Option<String> {
        if self.prompt.trim().is_empty() {
            return None;
        }
        let query = self.prompt.clone();
        self.history.push(query.clone());
        self.history_index = None;
        self.clear_prompt();
        self.error = None;
        self.query_start = Some(Instant::now());
        Some(query)
    }

    pub fn set_sql(&mut self, sql: String) {
        self.risk = Some(RiskLevel::from_sql(&sql));
        self.confidence = Some(92); // TODO: get from AI response
        self.sql = Some(sql);
        self.sql_status = Some("pending".to_string());
        self.explain_result = None; // clear old explain
        self.show_explain = false;
        self.log(LogLevel::Ok, "generated sql".to_string());
    }

    pub fn toggle_explain(&mut self) {
        self.show_explain = !self.show_explain;
    }

    pub fn copy_sql(&self) -> Option<String> {
        self.sql.clone()
    }

    pub fn copy_output(&self) -> Option<String> {
        let result = self.result.as_ref()?;

        if result.rows.is_empty() {
            return Some("no rows".to_string());
        }

        let mut output = String::new();

        // calculate column widths
        let mut widths: Vec<usize> = result.columns.iter().map(|c| c.len()).collect();
        for row in &result.rows {
            for (i, val) in row.iter().enumerate() {
                let len = format_value(val).len();
                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }

        // header
        for (i, col) in result.columns.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&format!("{:width$}", col, width = widths[i]));
        }
        output.push('\n');

        // separator
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&"-".repeat(*w));
        }
        output.push('\n');

        // rows
        for row in &result.rows {
            for (i, val) in row.iter().enumerate() {
                if i > 0 {
                    output.push_str("  ");
                }
                let s = format_value(val);
                output.push_str(&format!("{:width$}", s, width = widths[i]));
            }
            output.push('\n');
        }

        Some(output)
    }

    pub fn copy_cell(&self, row: usize, col: usize) -> Option<String> {
        self.result.as_ref().and_then(|r| {
            r.rows.get(row).and_then(|row_data| {
                row_data.get(col).map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
            })
        })
    }

    pub fn export_csv(&self) -> Option<String> {
        let result = self.result.as_ref()?;
        let mut csv = result.columns.join(",");
        csv.push('\n');

        for row in &result.rows {
            let values: Vec<String> = row
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => {
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s.clone()
                        }
                    }
                    serde_json::Value::Null => String::new(),
                    _ => v.to_string(),
                })
                .collect();
            csv.push_str(&values.join(","));
            csv.push('\n');
        }

        Some(csv)
    }

    pub fn set_result(&mut self, result: QueryResult) {
        if let Some(start) = self.query_start.take() {
            self.latency_ms = Some(start.elapsed().as_millis() as u64);
        }
        self.sql_status = Some(format!("executed ({}ms)", self.latency_ms.unwrap_or(0)));
        self.result = Some(result);
        self.error = None;
        self.loading = false;
        self.result_scroll = 0;
        self.log(LogLevel::Ok, "executed query".to_string());
    }

    pub fn set_error(&mut self, err: String) {
        if let Some(start) = self.query_start.take() {
            self.latency_ms = Some(start.elapsed().as_millis() as u64);
        }
        self.sql_status = Some("failed".to_string());
        self.error = Some(err.clone());
        self.loading = false;
        self.reconnecting = false;
        self.log(LogLevel::Error, err);
    }

    pub fn scroll_up(&mut self) {
        match self.panel {
            Panel::Results => self.result_scroll = self.result_scroll.saturating_sub(1),
            Panel::Logs => self.log_scroll = self.log_scroll.saturating_sub(1),
            _ => {}
        }
    }

    pub fn scroll_down(&mut self) {
        match self.panel {
            Panel::Results => self.result_scroll += 1,
            Panel::Logs => self.log_scroll += 1,
            _ => {}
        }
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
