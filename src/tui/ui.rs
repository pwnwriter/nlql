// ui rendering

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::tui::app::{App, LogLevel, Mode, Panel, Popup, RiskLevel};
use crate::tui::ascii::NLQL_LOGO;
use crate::tui::theme::ThemeKind;

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;

    // clear with bg color
    frame.render_widget(Clear, frame.area());
    frame.render_widget(Block::default().style(theme.base()), frame.area());

    // main layout: header + content + footer
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // header with logo
            Constraint::Min(10),   // content
            Constraint::Length(1), // footer
        ])
        .split(frame.area());

    render_header(frame, app, main[0]);
    render_content(frame, app, main[1]);
    render_footer(frame, app, main[2]);

    // render popups on top
    match app.popup {
        Popup::Themes => render_theme_popup(frame, app),
        Popup::Confirm => render_confirm_popup(frame, app),
        Popup::Connection => render_connection_popup(frame, app),
        Popup::SetupDbType => render_setup_db_type_popup(frame, app),
        Popup::SetupDbDetails => render_setup_db_details_popup(frame, app),
        Popup::SetupProvider => render_setup_provider_popup(frame, app),
        Popup::SetupApiKey => render_setup_api_key_popup(frame, app),
        Popup::None => {}
    }
}

fn render_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .style(theme.base());

    frame.render_widget(block, area);

    // split header: logo on left, info on right
    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .margin(1)
        .split(area);

    // render ascii logo
    let logo_lines: Vec<Line> = NLQL_LOGO
        .iter()
        .map(|&line| Line::styled(line, theme.accent()))
        .collect();

    let logo = Paragraph::new(logo_lines).style(theme.base());
    frame.render_widget(logo, inner[0]);

    // render info panel
    let latency = app
        .latency_ms
        .map(|ms| format!("{}ms", ms))
        .unwrap_or_else(|| "-".to_string());

    let cache = if app.cache_enabled { "ON" } else { "OFF" };

    let mode_str = match app.mode {
        Mode::Normal => "normal",
        Mode::Insert => "insert",
    };

    let info_lines = vec![
        Line::from(vec![
            Span::styled("| ", theme.muted()),
            Span::styled("nlql", theme.accent()),
        ]),
        Line::from(vec![
            Span::styled("| DB: ", theme.muted()),
            Span::styled(&app.db_info.database, theme.base()),
            Span::styled("  | Agent: ", theme.muted()),
            Span::styled(&app.agent_info.name, theme.base()),
            Span::styled(format!(" ({})", app.agent_info.model), theme.muted()),
            Span::styled(" | ", theme.muted()),
            Span::styled(&latency, theme.accent()),
            Span::styled(" | Cache ", theme.muted()),
            Span::styled(cache, theme.base()),
        ]),
        Line::from(vec![
            Span::styled("| Mode: ", theme.muted()),
            Span::styled(mode_str, theme.accent()),
        ]),
        Line::from(vec![
            Span::styled("| ", theme.muted()),
            Span::styled("[Tab]", theme.accent()),
            Span::styled(" Panels  ", theme.muted()),
            Span::styled("[t]", theme.accent()),
            Span::styled(" Themes  ", theme.muted()),
            Span::styled("[q]", theme.accent()),
            Span::styled(" Quit", theme.muted()),
        ]),
    ];

    let info = Paragraph::new(info_lines).style(theme.base());
    frame.render_widget(info, inner[1]);
}

fn render_content(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.fullscreen {
        // render only the active panel in fullscreen
        match app.panel {
            Panel::Prompt => render_prompt(frame, app, area),
            Panel::Sql => render_sql(frame, app, area),
            Panel::Results => render_results(frame, app, area),
            Panel::Logs => render_logs(frame, app, area),
        }
        return;
    }

    // 2x2 grid
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rows[0]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_prompt(frame, app, top_cols[0]);
    render_sql(frame, app, top_cols[1]);
    render_results(frame, app, bottom_cols[0]);
    render_logs(frame, app, bottom_cols[1]);
}

fn render_footer(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;

    let mut parts = vec![
        Span::styled(" Enter ", theme.base().bg(theme.accent).fg(theme.bg)),
        Span::styled(" Run ", theme.muted()),
        Span::styled("| ", theme.border()),
        Span::styled("y ", theme.accent()),
        Span::styled("SQL ", theme.muted()),
        Span::styled("Y ", theme.accent()),
        Span::styled("Output ", theme.muted()),
        Span::styled("| ", theme.border()),
        Span::styled("e ", theme.accent()),
        Span::styled("Explain ", theme.muted()),
        Span::styled("| ", theme.border()),
        Span::styled("f ", theme.accent()),
    ];

    if app.fullscreen {
        parts.push(Span::styled("Exit Full ", theme.warning()));
    } else {
        parts.push(Span::styled("Full ", theme.muted()));
    }

    parts.extend([
        Span::styled("| ", theme.border()),
        Span::styled("x ", theme.accent()),
        Span::styled("Export ", theme.muted()),
        Span::styled("| ", theme.border()),
        Span::styled("c ", theme.accent()),
        Span::styled("Connect ", theme.muted()),
        Span::styled("| ", theme.border()),
        Span::styled("t ", theme.accent()),
        Span::styled("Theme ", theme.muted()),
        Span::styled("| ", theme.border()),
        Span::styled("q ", theme.accent()),
        Span::styled("Quit ", theme.muted()),
    ]);

    let line = Line::from(parts);
    let paragraph = Paragraph::new(line)
        .style(theme.base())
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_prompt(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let active = app.panel == Panel::Prompt;

    let border_style = if active {
        theme.accent()
    } else {
        theme.border()
    };

    let title = " Prompt (Natural Language) ";

    let block = Block::default()
        .title(Span::styled(title, theme.title()))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.base());

    // render prompt - no visual cursor, we'll use the real terminal cursor
    let content = if app.prompt.is_empty() && app.mode != Mode::Insert {
        vec![Line::styled(
            "press 'i' to enter your query...",
            theme.muted(),
        )]
    } else {
        app.prompt
            .lines()
            .map(|l| Line::styled(l.to_string(), theme.base()))
            .collect()
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(theme.base())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // set cursor position when in insert mode
    if app.mode == Mode::Insert && active {
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        // calculate cursor position within text
        let (cursor_line, cursor_col) = {
            let mut line = 0usize;
            let mut col = 0usize;
            for (i, ch) in app.prompt.chars().enumerate() {
                if i >= app.prompt_cursor {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }
            (line, col)
        };

        let cursor_x = inner.x + cursor_col as u16;
        let cursor_y = inner.y + cursor_line as u16;

        // only set cursor if within bounds
        if cursor_x < inner.right() && cursor_y < inner.bottom() {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn render_sql(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let active = app.panel == Panel::Sql;

    let border_style = if active {
        theme.accent()
    } else {
        theme.border()
    };

    let block = Block::default()
        .title(Span::styled(" SQL + Execution ", theme.title()))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.base());

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Agent: ", theme.muted()),
            Span::styled(&app.agent_info.name, theme.base()),
        ]),
        Line::from(vec![
            Span::styled("Model: ", theme.muted()),
            Span::styled(&app.agent_info.model, theme.base()),
        ]),
    ];

    // confidence and risk
    if let Some(confidence) = app.confidence {
        let risk_style = match app.risk {
            Some(RiskLevel::Safe) => theme.success(),
            Some(RiskLevel::Moderate) => theme.warning(),
            Some(RiskLevel::Danger) => theme.error(),
            None => theme.muted(),
        };

        let risk_label = app.risk.map(|r| r.label()).unwrap_or("-");
        let sql_type = app
            .sql
            .as_ref()
            .map(|s| app.risk.map(|r| r.sql_type(s)).unwrap_or("-"))
            .unwrap_or("-");

        lines.push(Line::from(vec![
            Span::styled("Confidence: ", theme.muted()),
            Span::styled(format!("{}%", confidence), theme.accent()),
            Span::styled("  | Risk: ", theme.muted()),
            Span::styled(format!("{} ({})", risk_label, sql_type), risk_style),
        ]));
    }

    lines.push(Line::styled(
        "-----------------------------------------------",
        theme.border(),
    ));

    if app.loading {
        lines.push(Line::styled("generating sql...", theme.muted()));
    } else if let Some(sql) = &app.sql {
        for sql_line in sql.lines() {
            lines.push(Line::styled(sql_line.to_string(), theme.accent()));
        }
        lines.push(Line::from(""));

        // status line
        if let Some(status) = &app.sql_status {
            let (icon, status_style) = if status.contains("failed") {
                ("x", theme.error())
            } else if status.contains("executed") {
                ("+", theme.success())
            } else {
                ("~", theme.muted())
            };

            lines.push(Line::from(vec![
                Span::styled("Status: ", theme.muted()),
                Span::styled(format!("[{}] ", icon), status_style),
                Span::styled(status, status_style),
            ]));
        }
    } else {
        lines.push(Line::styled("no sql generated yet", theme.muted()));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(theme.base())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_results(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let active = app.panel == Panel::Results;

    let border_style = if active {
        theme.accent()
    } else {
        theme.border()
    };

    let title = match &app.result {
        Some(r) => format!(" Results ({} rows) ", r.row_count),
        None => " Results ".to_string(),
    };

    let block = Block::default()
        .title(Span::styled(title, theme.title()))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.base());

    // calculate available width (area - borders - padding)
    let available_width = area.width.saturating_sub(4) as usize;

    let content = if app.reconnecting {
        vec![Line::styled("reconnecting...", theme.muted())]
    } else if let Some(err) = &app.error {
        vec![Line::styled(format!("error: {err}"), theme.error())]
    } else if let Some(result) = &app.result {
        format_result(result, theme, available_width)
    } else {
        vec![Line::styled("run a query to see results", theme.muted())]
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(theme.base())
        .scroll((app.result_scroll as u16, 0));

    frame.render_widget(paragraph, area);
}

fn render_logs(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let active = app.panel == Panel::Logs;

    let border_style = if active {
        theme.accent()
    } else {
        theme.border()
    };

    let title = if app.show_explain {
        " Explain "
    } else {
        " Logs "
    };

    let block = Block::default()
        .title(Span::styled(title, theme.title()))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.base());

    let lines: Vec<Line> = if app.show_explain {
        if let Some(explain) = &app.explain_result {
            explain
                .lines()
                .map(|l| Line::styled(l.to_string(), theme.base()))
                .collect()
        } else {
            vec![
                Line::styled("press (e) to toggle EXPLAIN", theme.muted()),
                Line::styled("requires executing a query first", theme.muted()),
            ]
        }
    } else {
        let mut log_lines: Vec<Line> = app
            .logs
            .iter()
            .map(|entry| {
                let (prefix, style) = match entry.level {
                    LogLevel::Ok => ("[OK]", theme.success()),
                    LogLevel::Info => ("[--]", theme.muted()),
                    LogLevel::Warn => ("[!!]", theme.warning()),
                    LogLevel::Error => ("[ERR]", theme.error()),
                };
                Line::from(vec![
                    Span::styled(format!("{} ", prefix), style),
                    Span::styled(&entry.message, theme.base()),
                ])
            })
            .collect();

        log_lines.push(Line::from(""));
        log_lines.push(Line::from(vec![
            Span::styled("Press ", theme.muted()),
            Span::styled("(e)", theme.accent()),
            Span::styled(" to toggle EXPLAIN", theme.muted()),
        ]));

        log_lines
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(theme.base())
        .scroll((app.log_scroll as u16, 0));

    frame.render_widget(paragraph, area);
}

fn render_theme_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(40, 70, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" select theme ", theme.title()))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    let lines: Vec<Line> = ThemeKind::ALL
        .iter()
        .enumerate()
        .map(|(i, &kind)| {
            let name = kind.name();
            let is_selected = i == app.theme_scroll;

            if is_selected {
                Line::from(vec![
                    Span::styled(" > ", theme.accent()),
                    Span::styled(name, theme.selected().fg(theme.accent)),
                ])
            } else {
                Line::from(vec![Span::styled(format!("   {name}"), theme.base())])
            }
        })
        .collect();

    let help = Line::from(vec![
        Span::styled(" j/k ", theme.accent()),
        Span::styled("navigate  ", theme.muted()),
        Span::styled("enter ", theme.accent()),
        Span::styled("select  ", theme.muted()),
        Span::styled("esc ", theme.accent()),
        Span::styled("close", theme.muted()),
    ]);

    let mut all_lines = lines;
    all_lines.push(Line::from(""));
    all_lines.push(help);

    let paragraph = Paragraph::new(all_lines).block(block).style(theme.base());
    frame.render_widget(paragraph, area);
}

fn render_confirm_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(70, 50, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" confirm sql ", theme.title()))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    let mut lines = vec![
        Line::styled("generated sql:", theme.muted()),
        Line::from(""),
    ];

    if let Some(sql) = &app.sql {
        for sql_line in sql.lines() {
            lines.push(Line::styled(sql_line.to_string(), theme.accent()));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("execute this query? ", theme.base()),
        Span::styled("[y]es ", theme.success()),
        Span::styled("[n]o", theme.error()),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_connection_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(70, 30, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" edit connection ", theme.title()))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    let lines = vec![
        Line::styled("database url:", theme.muted()),
        Line::from(""),
        Line::raw(&app.connection_input),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("enter ", theme.accent()),
            Span::styled("connect  ", theme.muted()),
            Span::styled("esc ", theme.accent()),
            Span::styled("cancel  ", theme.muted()),
            Span::styled("ctrl+u ", theme.accent()),
            Span::styled("clear", theme.muted()),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(theme.base())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // set cursor position in connection input
    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let cursor_x = inner.x + app.connection_cursor as u16;
    let cursor_y = inner.y + 2; // line 3 (0-indexed: database url, empty, input)

    if cursor_x < inner.right() {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_setup_db_type_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(50, 40, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" nlql setup - database type ", theme.title()))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    use crate::tui::app::DbType;

    let mut lines = vec![
        Line::styled("select your database type:", theme.muted()),
        Line::from(""),
    ];

    for (i, db_type) in DbType::ALL.iter().enumerate() {
        let is_selected = i == app.setup_db_type_index;
        if is_selected {
            lines.push(Line::from(vec![
                Span::styled(" > ", theme.accent()),
                Span::styled(db_type.name(), theme.selected().fg(theme.accent)),
            ]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                format!("   {}", db_type.name()),
                theme.base(),
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("j/k ", theme.accent()),
        Span::styled("navigate  ", theme.muted()),
        Span::styled("enter ", theme.accent()),
        Span::styled("select  ", theme.muted()),
        Span::styled("esc ", theme.accent()),
        Span::styled("quit", theme.muted()),
    ]));

    let paragraph = Paragraph::new(lines).block(block).style(theme.base());
    frame.render_widget(paragraph, area);
}

fn render_setup_db_details_popup(frame: &mut Frame, app: &mut App) {
    use crate::tui::app::DbType;

    match app.setup_db_type {
        DbType::SQLite => render_setup_sqlite_popup(frame, app),
        _ => render_setup_server_db_popup(frame, app),
    }
}

fn render_setup_sqlite_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(70, 40, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" nlql setup - sqlite ", theme.title()))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    let mut lines = vec![
        Line::styled("enter the path to your sqlite database:", theme.muted()),
        Line::from(""),
        Line::from(vec![
            Span::styled("file: ", theme.accent()),
            Span::raw(&app.setup_db_file),
        ]),
        Line::from(""),
    ];

    if let Some(err) = &app.setup_error {
        lines.push(Line::styled(format!("error: {}", err), theme.error()));
        lines.push(Line::from(""));
    }

    lines.push(Line::styled("examples:", theme.muted()));
    lines.push(Line::styled("  ./mydata.db", theme.muted()));
    lines.push(Line::styled("  /path/to/database.sqlite", theme.muted()));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("enter ", theme.accent()),
        Span::styled("connect  ", theme.muted()),
        Span::styled("esc ", theme.accent()),
        Span::styled("back  ", theme.muted()),
        Span::styled("ctrl+u ", theme.accent()),
        Span::styled("clear", theme.muted()),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(theme.base())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // cursor position
    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let cursor_x = inner.x + 6 + app.setup_db_file_cursor as u16; // "file: " = 6 chars
    let cursor_y = inner.y + 2;

    if cursor_x < inner.right() {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_setup_server_db_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(70, 55, frame.area());

    frame.render_widget(Clear, area);

    let db_name = app.setup_db_type.name().to_lowercase();
    let block = Block::default()
        .title(Span::styled(
            format!(" nlql setup - {} ", db_name),
            theme.title(),
        ))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    let field_style = |field_idx: usize| {
        if app.setup_db_field == field_idx {
            theme.accent()
        } else {
            theme.muted()
        }
    };

    let field_label = |field_idx: usize, label: &str| {
        if app.setup_db_field == field_idx {
            Span::styled(format!("> {}: ", label), theme.accent())
        } else {
            Span::styled(format!("  {}: ", label), theme.muted())
        }
    };

    let masked_pass: String = "*".repeat(app.setup_db_pass.len());

    let mut lines = vec![
        Line::styled("enter connection details:", theme.muted()),
        Line::styled("(tab/shift+tab to switch fields)", theme.muted()),
        Line::from(""),
        Line::from(vec![
            field_label(0, "host"),
            Span::styled(&app.setup_db_host, field_style(0)),
        ]),
        Line::from(vec![
            field_label(1, "port"),
            Span::styled(&app.setup_db_port, field_style(1)),
        ]),
        Line::from(vec![
            field_label(2, "user"),
            Span::styled(&app.setup_db_user, field_style(2)),
        ]),
        Line::from(vec![
            field_label(3, "pass"),
            Span::styled(&masked_pass, field_style(3)),
        ]),
        Line::from(vec![
            field_label(4, "database"),
            Span::styled(&app.setup_db_name, field_style(4)),
        ]),
        Line::from(""),
    ];

    if let Some(err) = &app.setup_error {
        lines.push(Line::styled(format!("error: {}", err), theme.error()));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::styled("tab ", theme.accent()),
        Span::styled("next  ", theme.muted()),
        Span::styled("shift+tab ", theme.accent()),
        Span::styled("prev  ", theme.muted()),
        Span::styled("enter ", theme.accent()),
        Span::styled("connect", theme.muted()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("esc ", theme.accent()),
        Span::styled("back  ", theme.muted()),
        Span::styled("ctrl+u ", theme.accent()),
        Span::styled("clear field", theme.muted()),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(theme.base())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // cursor position based on active field
    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    let labels = ["host", "port", "user", "pass", "database"];
    let label_len = labels[app.setup_db_field].len() + 4; // "> " + ": "
    let cursor_offset = app.setup_db_get_cursor() as u16;
    let cursor_x = inner.x + label_len as u16 + cursor_offset;
    let cursor_y = inner.y + 3 + app.setup_db_field as u16; // 3 = header lines

    if cursor_x < inner.right() && cursor_y < inner.bottom() {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_setup_provider_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(50, 40, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" nlql setup - ai provider ", theme.title()))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    let providers = ["Claude (Anthropic)", "OpenAI (GPT-4)"];

    let mut lines = vec![
        Line::styled("select your ai provider:", theme.muted()),
        Line::from(""),
    ];

    for (i, provider) in providers.iter().enumerate() {
        let is_selected = i == app.setup_provider_index;
        if is_selected {
            lines.push(Line::from(vec![
                Span::styled(" > ", theme.accent()),
                Span::styled(*provider, theme.selected().fg(theme.accent)),
            ]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                format!("   {}", provider),
                theme.base(),
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("j/k ", theme.accent()),
        Span::styled("navigate  ", theme.muted()),
        Span::styled("enter ", theme.accent()),
        Span::styled("select  ", theme.muted()),
        Span::styled("esc ", theme.accent()),
        Span::styled("quit", theme.muted()),
    ]));

    let paragraph = Paragraph::new(lines).block(block).style(theme.base());
    frame.render_widget(paragraph, area);
}

fn render_setup_api_key_popup(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    let area = centered_rect(70, 45, frame.area());

    frame.render_widget(Clear, area);

    let provider_name = match app.setup_provider {
        crate::Provider::Claude => "claude",
        crate::Provider::OpenAI => "openai",
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" nlql setup - {} api key ", provider_name),
            theme.title(),
        ))
        .borders(Borders::ALL)
        .border_style(theme.accent())
        .style(theme.base());

    // mask the api key
    let masked: String = "*".repeat(app.setup_api_key_input.len());

    let mut lines = vec![
        Line::styled(format!("enter your {} api key:", provider_name), theme.muted()),
        Line::from(""),
        Line::raw(&masked),
        Line::from(""),
    ];

    // show error if any
    if let Some(err) = &app.setup_error {
        lines.push(Line::styled(format!("error: {}", err), theme.error()));
        lines.push(Line::from(""));
    }

    let env_var = match app.setup_provider {
        crate::Provider::Claude => "ANTHROPIC_API_KEY",
        crate::Provider::OpenAI => "OPENAI_API_KEY",
    };

    lines.push(Line::styled(
        format!("tip: set {} env var to skip this step", env_var),
        theme.muted(),
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("enter ", theme.accent()),
        Span::styled("continue  ", theme.muted()),
        Span::styled("esc ", theme.accent()),
        Span::styled("quit  ", theme.muted()),
        Span::styled("ctrl+u ", theme.accent()),
        Span::styled("clear", theme.muted()),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(theme.base())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // set cursor position
    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let cursor_x = inner.x + app.setup_api_key_cursor as u16;
    let cursor_y = inner.y + 2;

    if cursor_x < inner.right() {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_result(
    result: &crate::core::QueryResult,
    theme: &crate::tui::theme::Theme,
    available_width: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if result.rows.is_empty() {
        lines.push(Line::styled("no rows".to_string(), theme.muted()));
        return lines;
    }

    let num_cols = result.columns.len();
    if num_cols == 0 {
        return lines;
    }

    // calculate ideal column widths based on content
    let mut widths: Vec<usize> = result.columns.iter().map(|c| c.len()).collect();
    for row in &result.rows {
        for (i, val) in row.iter().enumerate() {
            if i < widths.len() {
                let len = format_value(val).len();
                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }
    }

    // calculate total width needed (columns + 1 space between each)
    let spacing = num_cols.saturating_sub(1); // spaces between columns
    let total_needed: usize = widths.iter().sum::<usize>() + spacing;

    // if too wide, shrink columns proportionally
    if total_needed > available_width && available_width > spacing {
        let content_width = available_width - spacing;
        let total_content: usize = widths.iter().sum();

        if total_content > 0 {
            // shrink proportionally, with minimum width of 4
            for w in &mut widths {
                *w = (*w * content_width / total_content).max(4);
            }
        }
    }

    // cap individual columns at reasonable max
    let max_col_width = (available_width / num_cols).max(8).min(30);
    for w in &mut widths {
        if *w > max_col_width {
            *w = max_col_width;
        }
    }

    // header
    let header: Vec<Span> = result
        .columns
        .iter()
        .enumerate()
        .flat_map(|(i, c)| {
            let w = widths.get(i).copied().unwrap_or(10);
            let s = truncate_str(c, w);
            let mut spans = vec![Span::styled(
                format!("{:width$}", s, width = w),
                ratatui::style::Style::default().fg(theme.accent),
            )];
            if i < num_cols - 1 {
                spans.push(Span::raw(" "));
            }
            spans
        })
        .collect();
    lines.push(Line::from(header));

    // separator
    let sep: String = widths
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let mut s = "-".repeat(*w);
            if i < num_cols - 1 {
                s.push(' ');
            }
            s
        })
        .collect();
    lines.push(Line::styled(
        sep,
        ratatui::style::Style::default().fg(theme.border),
    ));

    // rows
    for row in &result.rows {
        let cells: Vec<Span> = row
            .iter()
            .enumerate()
            .flat_map(|(i, v)| {
                let w = widths.get(i).copied().unwrap_or(10);
                let s = format_value(v);
                let s = truncate_str(&s, w);
                let mut spans = vec![Span::raw(format!("{:width$}", s, width = w))];
                if i < num_cols - 1 {
                    spans.push(Span::raw(" "));
                }
                spans
            })
            .collect();
        lines.push(Line::from(cells));
    }

    lines
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
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

impl crate::tui::theme::Theme {
    pub fn warning(&self) -> ratatui::style::Style {
        ratatui::style::Style::default().fg(self.warning)
    }
}
