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

use crate::{Ai, Db, Error, Provider};
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
    db: Option<Db>,
    schema: Option<String>,
    db_info: Option<DbInfo>,
    confirm: bool,
    provider: Provider,
    api_key: Option<String>,
) -> Result<(), Error> {
    // setup terminal
    enable_raw_mode().map_err(|e| Error::Server(e.to_string()))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| Error::Server(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| Error::Server(e.to_string()))?;

    // run app
    let result = run_app(&mut terminal, db, schema, db_info, confirm, provider, api_key).await;

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
    db: Option<Db>,
    schema: Option<String>,
    db_info: Option<DbInfo>,
    confirm: bool,
    provider: Provider,
    api_key: Option<String>,
) -> Result<(), Error> {
    // determine if we're in setup mode
    let setup_mode = db.is_none();

    // create app state
    let mut app = if setup_mode {
        App::new_setup()
    } else {
        App::new(
            schema.clone().unwrap_or_default(),
            db_info.clone().unwrap(),
            confirm,
        )
    };

    // these will be initialized after setup or immediately if db provided
    let mut ai: Option<Ai> = if !setup_mode {
        Some(Ai::new(provider, api_key.clone())?)
    } else {
        None
    };

    let db_arc: Arc<Mutex<Option<Db>>> = Arc::new(Mutex::new(db));
    let mut current_schema = schema.unwrap_or_default();

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
            .draw(|frame| ui::render(frame, &mut app))
            .map_err(|e| Error::Server(e.to_string()))?;

        // poll events
        if let Some(event) =
            poll_event(Duration::from_millis(100)).map_err(|e| Error::Server(e.to_string()))?
        {
            match handle_event(&mut app, event) {
                Action::Quit => break,
                Action::Submit(query) => {
                    // only process if we have AI initialized
                    if let Some(ref ai_client) = ai {
                        app.loading = true;
                        app.log(
                            LogLevel::Info,
                            format!("processing: {}", query.lines().next().unwrap_or(&query)),
                        );

                        // render loading state
                        terminal
                            .draw(|frame| ui::render(frame, &mut app))
                            .map_err(|e| Error::Server(e.to_string()))?;

                        // generate sql
                        match ai_client.generate_sql(&query, &current_schema).await {
                            Ok(sql) => {
                                app.set_sql(sql.clone());

                                if app.confirm_before_run {
                                    // show confirmation popup
                                    app.loading = false;
                                    app.show_confirm(sql);
                                } else {
                                    // execute directly
                                    terminal
                                        .draw(|frame| ui::render(frame, &mut app))
                                        .map_err(|e| Error::Server(e.to_string()))?;

                                    let db_guard = db_arc.lock().await;
                                    if let Some(ref db_conn) = *db_guard {
                                        match db_conn.execute(&sql).await {
                                            Ok(result) => app.set_result(result),
                                            Err(e) => app.set_error(e.to_string()),
                                        }
                                    }
                                }
                            }
                            Err(e) => app.set_error(e.to_string()),
                        }
                    }
                }
                Action::ConfirmSql => {
                    if let Some(sql) = app.confirm_sql() {
                        app.loading = true;

                        // render loading state
                        terminal
                            .draw(|frame| ui::render(frame, &mut app))
                            .map_err(|e| Error::Server(e.to_string()))?;

                        // execute
                        let db_guard = db_arc.lock().await;
                        if let Some(ref db_conn) = *db_guard {
                            match db_conn.execute(&sql).await {
                                Ok(result) => app.set_result(result),
                                Err(e) => app.set_error(e.to_string()),
                            }
                        }
                    }
                }
                Action::CancelSql => {
                    app.log(LogLevel::Info, "query cancelled".to_string());
                }
                Action::ToggleExplain => {
                    // run EXPLAIN if we have SQL and toggled to show explain
                    if app.show_explain && app.explain_result.is_none() {
                        if let Some(sql) = &app.sql {
                            let explain_sql = format!("EXPLAIN {}", sql);
                            let db_guard = db_arc.lock().await;
                            if let Some(ref db_conn) = *db_guard {
                                match db_conn.execute(&explain_sql).await {
                                    Ok(result) => {
                                        // format explain result as text
                                        let explain_text = result
                                            .rows
                                            .iter()
                                            .map(|row| {
                                                row.iter()
                                                    .map(|v| match v {
                                                        serde_json::Value::String(s) => s.clone(),
                                                        _ => v.to_string(),
                                                    })
                                                    .collect::<Vec<_>>()
                                                    .join(" | ")
                                            })
                                            .collect::<Vec<_>>()
                                            .join("\n");
                                        app.explain_result = Some(explain_text);
                                    }
                                    Err(e) => {
                                        app.explain_result = Some(format!("EXPLAIN failed: {}", e));
                                    }
                                }
                            }
                        }
                    }
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
                        .draw(|frame| ui::render(frame, &mut app))
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
                                *db_arc.lock().await = Some(new_db);
                            }
                            Err(e) => app.set_error(format!("schema error: {e}")),
                        },
                        Err(e) => app.set_error(format!("connection failed: {e}")),
                    }
                }
                Action::SetupConnectDb(url) => {
                    app.loading = true;
                    app.log(LogLevel::Info, "connecting to database...".to_string());

                    // render loading state
                    terminal
                        .draw(|frame| ui::render(frame, &mut app))
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
                                current_schema = new_schema;
                                app.db_info = new_info;
                                app.loading = false;
                                // move to provider selection
                                app.popup = app::Popup::SetupProvider;
                                *db_arc.lock().await = Some(new_db);
                                app.log(
                                    LogLevel::Ok,
                                    format!("connected to {}", app.db_info.dialect),
                                );
                            }
                            Err(e) => {
                                app.loading = false;
                                app.setup_set_error(format!("schema error: {e}"));
                            }
                        },
                        Err(e) => {
                            app.loading = false;
                            app.setup_set_error(format!("connection failed: {e}"));
                        }
                    }
                }
                Action::SetupComplete {
                    provider: setup_provider,
                    api_key: setup_api_key,
                } => {
                    // initialize AI client
                    let api_key_from_env = setup_api_key.is_none();
                    match Ai::new(setup_provider, setup_api_key) {
                        Ok(ai_client) => {
                            ai = Some(ai_client);
                            // finish setup and enter normal mode
                            app.finish_setup(app.db_info.clone(), &current_schema);
                            app.confirm_before_run = confirm;
                            if api_key_from_env {
                                app.log(
                                    LogLevel::Info,
                                    "using api key from environment".to_string(),
                                );
                            }
                        }
                        Err(e) => {
                            app.setup_set_error(format!("ai init failed: {e}"));
                        }
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
