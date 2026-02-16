// terminal ui

mod app;
mod ascii;
mod event;
mod theme;
mod ui;

pub use app::{App, DbInfo};
pub use theme::ThemeKind;

use crossterm::{
    cursor::SetCursorStyle,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, stdout};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::{Claude, Db, Error};
use app::{LogLevel, Mode};
use event::{Action, handle_event, poll_event};

fn copy_to_clipboard(text: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // try pbcopy (macOS)
    if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn()
        && let Some(stdin) = child.stdin.as_mut()
            && stdin.write_all(text.as_bytes()).is_ok() {
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }

    // try xclip (Linux)
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
        && let Some(stdin) = child.stdin.as_mut()
            && stdin.write_all(text.as_bytes()).is_ok() {
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }

    // try xsel (Linux fallback)
    if let Ok(mut child) = Command::new("xsel")
        .args(["--clipboard", "--input"])
        .stdin(Stdio::piped())
        .spawn()
        && let Some(stdin) = child.stdin.as_mut()
            && stdin.write_all(text.as_bytes()).is_ok() {
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }

    false
}

pub async fn run(
    db: Db,
    schema: String,
    db_info: DbInfo,
    confirm: bool,
    api_key: Option<String>,
) -> Result<(), Error> {
    // setup terminal
    enable_raw_mode().map_err(|e| Error::Server(e.to_string()))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| Error::Server(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| Error::Server(e.to_string()))?;

    // run app
    let result = run_app(&mut terminal, db, schema, db_info, confirm, api_key).await;

    // restore terminal
    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        SetCursorStyle::DefaultUserShape,
        LeaveAlternateScreen
    )
    .ok();
    terminal.show_cursor().ok();

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    db: Db,
    schema: String,
    db_info: DbInfo,
    confirm: bool,
    api_key: Option<String>,
) -> Result<(), Error> {
    let claude = Claude::new(api_key)?;
    let mut app = App::new(schema.clone(), db_info, confirm);
    let db = Arc::new(Mutex::new(db));
    let mut current_schema = schema;

    let mut last_mode = app.mode;

    loop {
        // update cursor style before render
        if app.mode != last_mode {
            let cursor_style = match app.mode {
                Mode::Insert => SetCursorStyle::BlinkingBar, // beam cursor
                Mode::Normal => SetCursorStyle::BlinkingBlock, // block cursor
            };
            execute!(terminal.backend_mut(), cursor_style).ok();
            last_mode = app.mode;
        }

        // render (cursor position is set in ui::render when in insert mode)
        terminal
            .draw(|frame| ui::render(frame, &app))
            .map_err(|e| Error::Server(e.to_string()))?;

        // poll events
        if let Some(event) =
            poll_event(Duration::from_millis(100)).map_err(|e| Error::Server(e.to_string()))?
        {
            match handle_event(&mut app, event) {
                Action::Quit => break,
                Action::Submit(query) => {
                    app.loading = true;
                    app.log(
                        LogLevel::Info,
                        format!("processing: {}", query.lines().next().unwrap_or(&query)),
                    );

                    // render loading state
                    terminal
                        .draw(|frame| ui::render(frame, &app))
                        .map_err(|e| Error::Server(e.to_string()))?;

                    // generate sql
                    match claude.generate_sql(&query, &current_schema).await {
                        Ok(sql) => {
                            app.set_sql(sql.clone());

                            if app.confirm_before_run {
                                // show confirmation popup
                                app.loading = false;
                                app.show_confirm(sql);
                            } else {
                                // execute directly
                                terminal
                                    .draw(|frame| ui::render(frame, &app))
                                    .map_err(|e| Error::Server(e.to_string()))?;

                                let db_guard = db.lock().await;
                                match db_guard.execute(&sql).await {
                                    Ok(result) => app.set_result(result),
                                    Err(e) => app.set_error(e.to_string()),
                                }
                            }
                        }
                        Err(e) => app.set_error(e.to_string()),
                    }
                }
                Action::ConfirmSql => {
                    if let Some(sql) = app.confirm_sql() {
                        app.loading = true;

                        // render loading state
                        terminal
                            .draw(|frame| ui::render(frame, &app))
                            .map_err(|e| Error::Server(e.to_string()))?;

                        // execute
                        let db_guard = db.lock().await;
                        match db_guard.execute(&sql).await {
                            Ok(result) => app.set_result(result),
                            Err(e) => app.set_error(e.to_string()),
                        }
                    }
                }
                Action::CancelSql => {
                    app.log(LogLevel::Info, "query cancelled".to_string());
                }
                Action::ToggleExplain => {
                    // explain mode toggled in app
                }
                Action::CopySql => {
                    if let Some(sql) = app.copy_sql() {
                        if copy_to_clipboard(&sql) {
                            app.log(LogLevel::Ok, "sql copied to clipboard".to_string());
                        } else {
                            app.log(LogLevel::Warn, "clipboard not available".to_string());
                        }
                    } else {
                        app.log(LogLevel::Warn, "no sql to copy".to_string());
                    }
                }
                Action::CopyOutput => {
                    if let Some(output) = app.copy_output() {
                        if copy_to_clipboard(&output) {
                            app.log(LogLevel::Ok, "output copied to clipboard".to_string());
                        } else {
                            app.log(LogLevel::Warn, "clipboard not available".to_string());
                        }
                    } else {
                        app.log(LogLevel::Warn, "no output to copy".to_string());
                    }
                }
                Action::ExportCsv => {
                    if let Some(csv) = app.export_csv() {
                        // write to file
                        let filename = format!(
                            "nlql_export_{}.csv",
                            chrono::Local::now().format("%Y%m%d_%H%M%S")
                        );
                        match std::fs::write(&filename, &csv) {
                            Ok(_) => app.log(LogLevel::Ok, format!("exported to {}", filename)),
                            Err(e) => app.log(LogLevel::Error, format!("export failed: {}", e)),
                        }
                    } else {
                        app.log(LogLevel::Warn, "no results to export".to_string());
                    }
                }
                Action::Reconnect(url) => {
                    app.reconnecting = true;
                    app.log(LogLevel::Info, "reconnecting...".to_string());

                    // render reconnecting state
                    terminal
                        .draw(|frame| ui::render(frame, &app))
                        .map_err(|e| Error::Server(e.to_string()))?;

                    // try to connect
                    match Db::connect(&url).await {
                        Ok(new_db) => match new_db.schema().await {
                            Ok(new_schema) => {
                                let tables = new_schema.matches("TABLE ").count();
                                let new_info = DbInfo {
                                    dialect: new_db.dialect_name().to_string(),
                                    host: new_db.host().to_string(),
                                    database: new_db.database().to_string(),
                                    tables,
                                    url: url.clone(),
                                };
                                current_schema = new_schema.clone();
                                app.update_db_info(new_info, new_schema);
                                *db.lock().await = new_db;
                            }
                            Err(e) => app.set_error(format!("schema error: {e}")),
                        },
                        Err(e) => app.set_error(format!("connection failed: {e}")),
                    }
                }
                Action::None => {}
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
