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
                InputMode::BoardSelection => {
                    match key.code {
                        // Change this from quitting to returning to Normal mode
                        KeyCode::Esc => {
                            // Only return to Normal mode if we're not in the initial app startup
                            if app.columns.len() > 0 {
                                app.input_mode = InputMode::Normal;
                            } else {
                                // If no board is loaded, Esc should still quit
                                return Ok(());
                            }
                        }
                        KeyCode::Char(' ') => {
                            app.space_pressed = true;
                        }
                        KeyCode::Char('b') if app.space_pressed => {
                            app.space_pressed = false;
                            // Return to normal mode
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('q') => return Ok(()), // Explicit quit option
                        KeyCode::Up | KeyCode::Char('k') => app.select_prev_board(),
                        KeyCode::Down | KeyCode::Char('j') => app.select_next_board(),
                        KeyCode::Enter => {
                            // Handle board selection
                            if let Err(e) = app.load_selected_board() {
                                eprintln!("Error loading board: {}", e);
                            }
                        }
                        _ => {
                            app.space_pressed = false;
                        }
                    }
                }
                InputMode::AddingBoard => {
                    match key.code {
                        KeyCode::Enter => {
                            let board_name = if app.input_text.is_empty() {
                                "My Kanban Board".to_string()
                            } else {
                                app.input_text.clone()
                            };

                            // Create the new board
                            if let Err(e) = app.create_new_board(&board_name) {
                                eprintln!("Error creating board: {}", e);
                            } else {
                                app.input_mode = InputMode::Normal;
                            }

                            app.input_text.clear();
                        }
                        KeyCode::Esc => {
                            // Return to board selection
                            app.input_mode = InputMode::BoardSelection;
                            app.input_text.clear();
                        }
                        KeyCode::Char(c) => app.input_text.push(c),
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        }
                        _ => {}
                    }
                }
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
                            KeyCode::Char(' ') => {
                                app.space_pressed = true;
                            }
                            KeyCode::Char('c') if app.space_pressed => {
                                app.space_pressed = false;
                                app.input_mode = InputMode::AddingColumn;
                            }
                            KeyCode::Char('j') if app.space_pressed => {
                                app.space_pressed = false;
                                app.input_mode = InputMode::JumpToColumnMode;
                            }

                            KeyCode::Char('t') if app.space_pressed => {
                                app.space_pressed = false;
                                app.input_mode = InputMode::AddingTask;
                            }
                            KeyCode::Char('d') if app.space_pressed => {
                                app.space_pressed = false;
                                app.delete_current_task();
                            }
                            KeyCode::Char('b') if app.space_pressed => {
                                app.space_pressed = false;
                                // Toggle board selection
                                if app.input_mode == InputMode::BoardSelection {
                                    // If already in board selection, return to normal mode
                                    app.input_mode = InputMode::Normal;
                                } else {
                                    // Otherwise scan boards and enter board selection mode
                                    if let Err(e) = app.scan_available_boards() {
                                        eprintln!("Error scanning boards: {}", e);
                                    }
                                    app.input_mode = InputMode::BoardSelection;
                                }
                            }

                            KeyCode::Char('g') => {
                                // Only enter column selection mode if there's a task selected in the current column
                                if let Some(column) = app.columns.get(app.active_column) {
                                    if column.selected_task.is_some() {
                                        app.input_mode = InputMode::ColumnSelectionMode;
                                    }
                                }
                            }
                            KeyCode::Char('h') => app.select_prev_column(),
                            // KeyCode::Char('k') if key.modifiers == KeyModifiers::CONTROL => {
                            //     app.input_mode = InputMode::MoveMode;
                            // }
                            KeyCode::Char('l') => app.select_next_column(),
                            KeyCode::Char('j') => app.select_next_task(),
                            KeyCode::Char('k') => app.select_prev_task(),
                            // Keep save functionality with Ctrl+S
                            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                                // Explicitly save board to file
                                if let Err(e) = app.save_board() {
                                    eprintln!("Error saving board: {}", e);
                                }
                            }
                            _ => {
                                app.space_pressed = false;
                            }
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
                InputMode::ColumnSelectionMode => match key.code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Char(c) if c >= '1' && c <= '9' => {
                        let index = c.to_digit(10).unwrap() as usize;
                        // Handle the column index: key 1 maps to index 0, key 2 to index 1, etc.
                        let target_index = index - 1;

                        // Only move if the target index is valid and not the current column
                        if target_index < app.columns.len() && target_index != app.active_column {
                            app.move_task_to_column(target_index);
                        }

                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::JumpToColumnMode => match key.code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Char(c) if c >= '1' && c <= '9' => {
                        let index = c.to_digit(10).unwrap() as usize;
                        // Map key 1 to index 0, key 2 to index 1, etc.
                        let target_index = index - 1;

                        // Only jump if the target index is valid
                        if target_index < app.columns.len() {
                            app.active_column = target_index;

                            // Clear selection in all non-active columns
                            for (i, column) in app.columns.iter_mut().enumerate() {
                                if i != app.active_column {
                                    column.selected_task = None;
                                }
                            }
                        }

                        app.input_mode = InputMode::Normal;
                    }
                    _ => app.input_mode = InputMode::Normal, // Any other key cancels the mode
                },
            }
        }
    }
}
