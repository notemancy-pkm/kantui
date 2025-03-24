use crate::kanban::models::{App, InputMode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};
use std::io;

/// Runs the main event loop for the application.
pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    let mut last_key: Option<KeyCode> = None;

    loop {
        terminal.draw(|f| crate::kanban::ui::render::draw_ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => {
                    if key.modifiers.is_empty() && key.code == KeyCode::Char('d') {
                        if let Some(KeyCode::Char('d')) = last_key {
                            last_key = None;
                            app.input_mode = InputMode::ConfirmDeleteColumn;
                            continue;
                        } else {
                            last_key = Some(KeyCode::Char('d'));
                            continue;
                        }
                    } else {
                        last_key = None;
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                                app.input_mode = InputMode::AddingColumn;
                            }
                            KeyCode::Char('h') => app.select_prev_column(),
                            KeyCode::Char('k') if key.modifiers == KeyModifiers::CONTROL => {
                                app.input_mode = InputMode::MoveMode;
                            }
                            KeyCode::Char('a') if key.modifiers == KeyModifiers::CONTROL => {
                                app.input_mode = InputMode::AddingTask;
                            }
                            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                                app.delete_current_task();
                            }
                            KeyCode::Char('l') => app.select_next_column(),
                            KeyCode::Char('j') => app.select_next_task(),
                            KeyCode::Char('k') => app.select_prev_task(),
                            _ => {}
                        }
                    }
                }
                InputMode::AddingColumn => match key.code {
                    KeyCode::Enter => {
                        let column_name = if app.input_text.is_empty() {
                            "New Column".to_string()
                        } else {
                            app.input_text.clone()
                        };
                        app.add_column(&column_name);
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input_text.clear();
                    }
                    KeyCode::Char(c) => app.input_text.push(c),
                    KeyCode::Backspace => {
                        app.input_text.pop();
                    }
                    _ => {}
                },
                InputMode::AddingTask => match key.code {
                    KeyCode::Enter => {
                        let task_name = if app.input_text.is_empty() {
                            "New Task".to_string()
                        } else {
                            app.input_text.clone()
                        };
                        app.add_task(&task_name);
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input_text.clear();
                    }
                    KeyCode::Char(c) => app.input_text.push(c),
                    KeyCode::Backspace => {
                        app.input_text.pop();
                    }
                    _ => {}
                },
                InputMode::MoveMode => match key.code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Char(c) if c >= '0' && c <= '9' => {
                        let index = c.to_digit(10).unwrap() as usize;
                        app.jump_to_column(index);
                    }
                    _ => {}
                },
                InputMode::ConfirmDeleteColumn => match key.code {
                    KeyCode::Char('y') => {
                        app.delete_current_column();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('n') => app.input_mode = InputMode::Normal,
                    _ => {}
                },
            }
        }
    }
}
