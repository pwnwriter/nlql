// event handling

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::tui::app::{App, Mode, Popup};

pub enum Action {
    None,
    Quit,
    Submit(String),
    ConfirmSql,
    CancelSql,
    Reconnect(String),
    ToggleExplain,
    CopySql,
    CopyOutput,
    ExportCsv,
}

pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Key(key) => handle_key(app, key),
        _ => Action::None,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // global keys (work in any mode)
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Action::Quit;
        }
        _ => {}
    }

    // handle popups first
    match app.popup {
        Popup::Themes => return handle_theme_popup(app, key),
        Popup::Confirm => return handle_confirm_popup(app, key),
        Popup::Connection => return handle_connection_popup(app, key),
        Popup::None => {}
    }

    match app.mode {
        Mode::Normal => handle_normal_key(app, key),
        Mode::Insert => handle_insert_key(app, key),
    }
}

fn handle_theme_popup(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.close_popup();
            Action::None
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.theme_scroll_down();
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.theme_scroll_up();
            Action::None
        }
        KeyCode::Enter => {
            app.select_theme();
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_confirm_popup(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmSql,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.cancel_sql();
            Action::CancelSql
        }
        _ => Action::None,
    }
}

fn handle_connection_popup(app: &mut App, key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('a') => {
                app.connection_move_start();
                Action::None
            }
            KeyCode::Char('e') => {
                app.connection_move_end();
                Action::None
            }
            KeyCode::Char('u') => {
                app.connection_clear();
                Action::None
            }
            _ => Action::None,
        };
    }

    match key.code {
        KeyCode::Esc => {
            app.close_popup();
            Action::None
        }
        KeyCode::Enter => {
            if let Some(url) = app.submit_connection() {
                Action::Reconnect(url)
            } else {
                Action::None
            }
        }
        KeyCode::Char(c) => {
            app.connection_insert_char(c);
            Action::None
        }
        KeyCode::Backspace => {
            app.connection_delete_char();
            Action::None
        }
        KeyCode::Delete => {
            app.connection_delete_char_forward();
            Action::None
        }
        KeyCode::Left => {
            app.connection_move_left();
            Action::None
        }
        KeyCode::Right => {
            app.connection_move_right();
            Action::None
        }
        KeyCode::Home => {
            app.connection_move_start();
            Action::None
        }
        KeyCode::End => {
            app.connection_move_end();
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        // quit
        KeyCode::Char('q') => Action::Quit,

        // enter insert mode
        KeyCode::Char('i') => {
            app.enter_insert();
            Action::None
        }
        KeyCode::Char('a') => {
            app.move_cursor_end();
            app.enter_insert();
            Action::None
        }
        KeyCode::Char('I') => {
            app.move_cursor_start();
            app.enter_insert();
            Action::None
        }
        KeyCode::Char('A') => {
            app.move_cursor_end();
            app.enter_insert();
            Action::None
        }

        // panel navigation
        KeyCode::Tab => {
            app.cycle_panel();
            Action::None
        }

        // theme popup
        KeyCode::Char('t') => {
            app.open_theme_popup();
            Action::None
        }

        // connection popup
        KeyCode::Char('c') => {
            app.open_connection_popup();
            Action::None
        }

        // explain toggle
        KeyCode::Char('e') => {
            app.toggle_explain();
            Action::ToggleExplain
        }

        // copy sql
        KeyCode::Char('y') => Action::CopySql,

        // copy output
        KeyCode::Char('Y') => Action::CopyOutput,

        // export csv
        KeyCode::Char('x') => Action::ExportCsv,

        // scrolling
        KeyCode::Char('j') | KeyCode::Down => {
            app.scroll_down();
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.scroll_up();
            Action::None
        }

        // history
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.history_up();
            Action::None
        }
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.history_down();
            Action::None
        }

        // submit
        KeyCode::Enter => {
            if let Some(query) = app.submit() {
                Action::Submit(query)
            } else {
                Action::None
            }
        }

        _ => Action::None,
    }
}

fn handle_insert_key(app: &mut App, key: KeyEvent) -> Action {
    // check control keys first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('a') => {
                app.move_cursor_start();
                Action::None
            }
            KeyCode::Char('e') => {
                app.move_cursor_end();
                Action::None
            }
            KeyCode::Char('u') => {
                app.clear_prompt();
                Action::None
            }
            KeyCode::Char('p') => {
                app.history_up();
                Action::None
            }
            KeyCode::Char('n') => {
                app.history_down();
                Action::None
            }
            KeyCode::Enter => {
                // ctrl+enter for newline
                app.insert_newline();
                Action::None
            }
            _ => Action::None,
        };
    }

    // shift+enter for newline
    if key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Enter {
        app.insert_newline();
        return Action::None;
    }

    match key.code {
        // exit insert mode
        KeyCode::Esc => {
            app.exit_insert();
            Action::None
        }

        // submit
        KeyCode::Enter => {
            app.exit_insert();
            if let Some(query) = app.submit() {
                Action::Submit(query)
            } else {
                Action::None
            }
        }

        // editing
        KeyCode::Char(c) => {
            app.insert_char(c);
            Action::None
        }
        KeyCode::Backspace => {
            app.delete_char();
            Action::None
        }
        KeyCode::Delete => {
            app.delete_char_forward();
            Action::None
        }

        // cursor movement
        KeyCode::Left => {
            app.move_cursor_left();
            Action::None
        }
        KeyCode::Right => {
            app.move_cursor_right();
            Action::None
        }
        KeyCode::Home => {
            app.move_cursor_start();
            Action::None
        }
        KeyCode::End => {
            app.move_cursor_end();
            Action::None
        }

        // history
        KeyCode::Up => {
            app.history_up();
            Action::None
        }
        KeyCode::Down => {
            app.history_down();
            Action::None
        }

        _ => Action::None,
    }
}
