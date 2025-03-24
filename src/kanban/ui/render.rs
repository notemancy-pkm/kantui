use crate::kanban::models::{App, InputMode};
use crate::kanban::ui::task_formatter::format_task_with_wrapping;
use ratatui::{
    Frame,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::io;

const COLUMN_WIDTH: u16 = 50;
const COLUMN_MARGIN: u16 = 2;

/// Draws the overall UI including the title, columns, tasks and any popups.
pub fn draw_ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Set the background color for the entire app
    let background = Block::default()
        .style(Style::default().bg(Color::Rgb(22, 22, 22))) // #161616
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Check if we need to show the board selection popup
    match app.input_mode {
        InputMode::BoardSelection => {
            draw_board_selection(f, app, size);
            return;
        }
        InputMode::AddingBoard => {
            draw_new_board_popup(f, app, size);
            return;
        }
        _ => {}
    }

    // Render the title.
    let title = Paragraph::new(app.title.clone())
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);
    f.render_widget(title, chunks[0]);

    // Determine layout for the columns.
    let available_width = chunks[1].width;
    let column_with_margin = COLUMN_WIDTH + (COLUMN_MARGIN * 2);
    let max_visible_columns = (available_width / column_with_margin).max(1) as usize;
    let start_idx = if app.columns.len() <= max_visible_columns {
        0
    } else if app.active_column >= app.scroll_offset + max_visible_columns {
        app.active_column + 1 - max_visible_columns
    } else if app.active_column < app.scroll_offset {
        app.active_column
    } else {
        app.scroll_offset
    };
    let visible_columns = max_visible_columns.min(app.columns.len() - start_idx);

    // Create layout constraints for each column.
    let mut column_constraints = Vec::new();
    for _ in 0..visible_columns {
        column_constraints.push(Constraint::Length(COLUMN_MARGIN)); // left margin
        column_constraints.push(Constraint::Length(COLUMN_WIDTH)); // column
        column_constraints.push(Constraint::Length(COLUMN_MARGIN)); // right margin
    }
    if !column_constraints.is_empty()
        && available_width > column_with_margin * (column_constraints.len() / 3) as u16
    {
        column_constraints.push(Constraint::Min(0));
    }
    let columns_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(column_constraints)
        .split(chunks[1]);

    // Render each visible column.
    for (layout_idx, column_idx) in (start_idx..app.columns.len())
        .enumerate()
        .take(visible_columns)
    {
        let column = &app.columns[column_idx];
        let column_area = columns_layout[layout_idx * 3 + 1]; // Skip left margin.
        let style = if column_idx == app.active_column {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let title_with_index = format!(
            "{} ({}/{})",
            column.title,
            column_idx + 1,
            app.columns.len()
        );
        let title_text = Paragraph::new(title_with_index)
            .alignment(Alignment::Center)
            .style(style);
        let horizontal_line = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Blue));
        let column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(column_area);

        f.render_widget(title_text, column_layout[0]);
        f.render_widget(horizontal_line, column_layout[1]);

        let tasks: Vec<ListItem> = column
            .tasks
            .iter()
            .enumerate()
            .flat_map(|(i, task)| {
                // Get jump label for this task if in jump task mode
                let jump_label = if app.input_mode == InputMode::JumpToTaskMode {
                    app.get_jump_label_for_task(column_idx, i)
                } else {
                    None
                };

                // Format task with optional jump label
                let formatted_task = format_task_with_wrapping(
                    task,
                    column_area.width,
                    jump_label,
                    app.input_mode == InputMode::JumpToTaskMode,
                );

                // Apply appropriate styling
                let style = if column.selected_task == Some(i) {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(Color::Rgb(38, 38, 38))
                };

                let task_item = ListItem::new(formatted_task).style(style);
                vec![task_item, ListItem::new("")]
            })
            .collect();

        let tasks_list = List::new(tasks).block(Block::default());
        f.render_widget(tasks_list, column_layout[2]);
    }

    // Render help text.
    let help_text = match app.input_mode {
        InputMode::Normal => {
            "Use 'h'/'l' to navigate columns | 'j'/'k' to navigate tasks | 'm' to move task | 'gc' to jump to column | 'gt' to jump to task | ac to add column | at to add task | dt to delete task | dc to delete column | b for board selection | Ctrl+S to save | 'q' to quit"
        }
        InputMode::AddingColumn => "Enter column name | Enter to confirm | Esc to cancel",
        InputMode::AddingTask => "Enter task name | Enter to confirm | Esc to cancel",
        InputMode::MoveMode => "Press 0-9 to jump to that column | Esc to cancel",
        InputMode::ConfirmDeleteColumn => "Press y to delete | n to cancel",
        InputMode::ColumnSelectionMode => {
            "Press number to move task to that column | Esc to cancel"
        }
        InputMode::JumpToColumnMode => "Press number to jump to that column | Esc to cancel",
        InputMode::JumpToTaskMode => {
            "Press colored key shown on a task to jump to it | Esc to cancel"
        }
        _ => "", // BoardSelection and AddingBoard are handled separately
    };
    let help = Paragraph::new(help_text)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(22, 22, 22)),
        ) // #161616 for bg
        .alignment(Alignment::Center);
    let help_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(size);
    f.render_widget(help, help_layout[1]);

    // Draw any popups.
    draw_popup(f, app, size);
}

/// Helper function to draw the board selection popup
fn draw_board_selection(f: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    // Clear the screen with background color
    let background = Block::default()
        .style(Style::default().bg(Color::Rgb(22, 22, 22))) // #161616
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Create a centered popup
    let popup_width = 60;
    let popup_height = std::cmp::min(20, app.available_boards.len() as u16 + 6);

    let popup_area = ratatui::layout::Rect::new(
        (size.width.saturating_sub(popup_width)) / 2,
        (size.height.saturating_sub(popup_height)) / 2,
        popup_width.min(size.width),
        popup_height.min(size.height),
    );

    // Create popup block
    let popup_block = Block::default()
        .title("Select Kanban Board")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Blue).bg(Color::Rgb(38, 38, 38))); // #262626 for popup bg

    f.render_widget(popup_block.clone(), popup_area);

    // Create layout for popup content
    let inner_area = popup_block.inner(popup_area);
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title and description
            Constraint::Min(1),    // List of boards
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

    // Render description
    let description = if app.available_boards.is_empty() {
        "No boards found. Create a new one."
    } else {
        "Select a board or create a new one:"
    };

    let desc_text = Paragraph::new(description)
        .style(Style::default())
        .alignment(Alignment::Center);

    f.render_widget(desc_text, popup_chunks[0]);

    // Render the list of boards
    if !app.available_boards.is_empty() {
        let board_items: Vec<ListItem> = app
            .available_boards
            .iter()
            .enumerate()
            .map(|(i, board_name)| {
                let style = if app.selected_board_index == Some(i) {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(82, 82, 82)) // #525252 for selection
                        .add_modifier(Modifier::BOLD)
                } else if i == app.available_boards.len() - 1 {
                    // Special style for "Create New Board" option
                    Style::default().fg(Color::Green).bg(Color::Rgb(38, 38, 38)) // #262626 for bg
                } else {
                    Style::default().bg(Color::Rgb(38, 38, 38)) // #262626 for bg
                };

                // Special formatting for "Create New Board" option
                let text = if i == app.available_boards.len() - 1 {
                    format!("➕ {}", board_name)
                } else {
                    format!("📋 {}", board_name)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let boards_list = List::new(board_items)
            .block(Block::default().style(Style::default().bg(Color::Rgb(38, 38, 38)))) // #262626 for block bg
            .highlight_style(Style::default().bg(Color::Rgb(82, 82, 82)).fg(Color::White)); // #525252 for highlight

        f.render_widget(boards_list, popup_chunks[1]);
    }

    // Render help text
    let help_text = "↑↓: Navigate | Enter: Select | Esc: Quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help, popup_chunks[2]);
}

/// Helper function to draw the new board creation popup
fn draw_new_board_popup(f: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    // Set background color
    let background = Block::default()
        .style(Style::default().bg(Color::Rgb(22, 22, 22))) // #161616
        .borders(Borders::NONE);
    f.render_widget(background, size);

    let popup_width = 60;
    let popup_height = 5;

    let popup_area = ratatui::layout::Rect::new(
        (size.width.saturating_sub(popup_width)) / 2,
        (size.height.saturating_sub(popup_height)) / 2,
        popup_width.min(size.width),
        popup_height.min(size.height),
    );

    // Clear the area first to ensure clean rendering
    f.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title("Create New Board")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(38, 38, 38))); // #262626 for popup bg

    f.render_widget(&popup_block, popup_area);

    let input_area = popup_block.inner(popup_area);

    let input = Paragraph::new(app.input_text.clone())
        .style(Style::default().bg(Color::Rgb(38, 38, 38))) // #262626 for input background
        .block(
            Block::default()
                .title("Enter board name:")
                .style(Style::default().bg(Color::Rgb(38, 38, 38))),
        ) // #262626 for input block
        .wrap(Wrap { trim: true });

    f.render_widget(input, input_area);

    // Position cursor for typing
    f.set_cursor_position(Position {
        x: input_area.x + app.input_text.len() as u16,
        y: input_area.y + 1, // +1 to account for the title
    });
}

/// Helper function to draw popups based on the input mode.
fn draw_popup(f: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    match app.input_mode {
        InputMode::BoardSelection | InputMode::AddingBoard => {
            // These are handled separately in draw_board_selection and draw_new_board_popup
        }
        InputMode::ColumnSelectionMode => {
            draw_column_selection_popup(f, app, size);
        }
        InputMode::AddingColumn => {
            let popup_width = 70;
            let popup_height = 5;
            let popup_area = ratatui::layout::Rect::new(
                (size.width.saturating_sub(popup_width)) / 2,
                (size.height.saturating_sub(popup_height)) / 2,
                popup_width.min(size.width),
                popup_height.min(size.height),
            );
            f.render_widget(Clear, popup_area);
            let popup_block = Block::default()
                .title("New Column")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(38, 38, 38))); // #262626 for popup bg
            f.render_widget(&popup_block, popup_area);
            let input_area = popup_block.inner(popup_area);
            let input = Paragraph::new(app.input_text.clone())
                .style(Style::default().bg(Color::Rgb(38, 38, 38))) // #262626 for input bg
                .block(Block::default()) // #262626 for block bg
                .wrap(Wrap { trim: true });
            f.render_widget(input, input_area);
            f.set_cursor_position(Position {
                x: input_area.x + app.input_text.len() as u16,
                y: input_area.y,
            });
        }
        InputMode::AddingTask => {
            let popup_width = 70;
            let popup_height = 5;
            let popup_area = ratatui::layout::Rect::new(
                (size.width.saturating_sub(popup_width)) / 2,
                (size.height.saturating_sub(popup_height)) / 2,
                popup_width.min(size.width),
                popup_height.min(size.height),
            );
            f.render_widget(Clear, popup_area);
            let popup_block = Block::default()
                .title("New Task")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(38, 38, 38))); // #262626 for popup bg
            f.render_widget(&popup_block, popup_area);
            let input_area = popup_block.inner(popup_area);
            let input = Paragraph::new(app.input_text.clone())
                .style(Style::default().bg(Color::Rgb(38, 38, 38))) // #262626 for input bg
                .block(Block::default()) // #262626 for block bg
                .wrap(Wrap { trim: true });
            f.render_widget(input, input_area);
            f.set_cursor_position(Position {
                x: input_area.x + app.input_text.len() as u16,
                y: input_area.y,
            });
        }
        InputMode::ConfirmDeleteColumn => {
            let popup_width = 50;
            let popup_height = 3;
            let popup_area = ratatui::layout::Rect::new(
                (size.width.saturating_sub(popup_width)) / 2,
                (size.height.saturating_sub(popup_height)) / 2,
                popup_width.min(size.width),
                popup_height.min(size.height),
            );
            f.render_widget(Clear, popup_area);
            let popup_block = Block::default()
                .title("Confirm Delete Column")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(38, 38, 38))); // #262626 for popup bg
            f.render_widget(&popup_block, popup_area);
            let inner = popup_block.inner(popup_area);
            let column_name = app
                .columns
                .get(app.active_column)
                .map(|col| col.title.as_str())
                .unwrap_or("");
            let text = Paragraph::new(format!("Delete column '{}' ? (y/n)", column_name))
                .style(Style::default().fg(Color::Red).bg(Color::Rgb(38, 38, 38))) // #262626 for text bg
                .alignment(Alignment::Center);
            f.render_widget(text, inner);
        }
        InputMode::JumpToColumnMode => {
            draw_jump_column_popup(f, app, size);
        }
        InputMode::Normal | InputMode::MoveMode | InputMode::JumpToTaskMode => {
            // No popups for these modes
        }
    }
}

fn draw_column_selection_popup(f: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    let popup_width = 50;
    let popup_height = std::cmp::min(app.columns.len() as u16 + 4, 15); // Max height of 15

    let popup_area = ratatui::layout::Rect::new(
        (size.width.saturating_sub(popup_width)) / 2,
        (size.height.saturating_sub(popup_height)) / 2,
        popup_width.min(size.width),
        popup_height.min(size.height),
    );

    f.render_widget(Clear, popup_area);
    let popup_block = Block::default()
        .title("Move Task to Column")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(38, 38, 38))); // #262626 for popup bg

    f.render_widget(&popup_block, popup_area);

    let inner = popup_block.inner(popup_area);

    // Calculate the available height for the column list
    let list_height = inner.height.saturating_sub(2);

    // Create a list of columns with their indices, starting from 1
    let list_items: Vec<ListItem> = app
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            // Format column index starting from 1
            let text = format!("{}: {}", i + 1, col.title);

            ListItem::new(text).style(if i == app.active_column {
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Rgb(38, 38, 38))
            } else {
                Style::default().bg(Color::Rgb(38, 38, 38))
            })
        })
        .collect();

    let columns_list = List::new(list_items)
        .block(Block::default().style(Style::default().bg(Color::Rgb(38, 38, 38))));

    let list_area = ratatui::layout::Rect::new(inner.x, inner.y, inner.width, list_height);

    f.render_widget(columns_list, list_area);

    // Instructions
    let instructions = Paragraph::new("Press a number to move task to that column, Esc to cancel")
        .style(Style::default().fg(Color::Gray).bg(Color::Rgb(38, 38, 38)))
        .alignment(Alignment::Center);

    let instructions_area =
        ratatui::layout::Rect::new(inner.x, inner.y + list_height, inner.width, 2);

    f.render_widget(instructions, instructions_area);
}

fn draw_jump_column_popup(f: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    let popup_width = 50;
    let popup_height = std::cmp::min(app.columns.len() as u16 + 4, 15); // Max height of 15

    let popup_area = ratatui::layout::Rect::new(
        (size.width.saturating_sub(popup_width)) / 2,
        (size.height.saturating_sub(popup_height)) / 2,
        popup_width.min(size.width),
        popup_height.min(size.height),
    );

    f.render_widget(Clear, popup_area);
    let popup_block = Block::default()
        .title("Jump to Column")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(38, 38, 38))); // #262626 for popup bg

    f.render_widget(&popup_block, popup_area);

    let inner = popup_block.inner(popup_area);

    // Calculate the available height for the column list
    let list_height = inner.height.saturating_sub(2);

    // Create a list of columns with their indices, starting from 1
    let list_items: Vec<ListItem> = app
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            // Format column index starting from 1
            let text = format!("{}: {}", i + 1, col.title);

            ListItem::new(text).style(if i == app.active_column {
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Rgb(38, 38, 38))
            } else {
                Style::default().bg(Color::Rgb(38, 38, 38))
            })
        })
        .collect();

    let columns_list = List::new(list_items)
        .block(Block::default().style(Style::default().bg(Color::Rgb(38, 38, 38))));

    let list_area = ratatui::layout::Rect::new(inner.x, inner.y, inner.width, list_height);

    f.render_widget(columns_list, list_area);

    // Instructions
    let instructions = Paragraph::new("Press a number to jump to that column, Esc to cancel")
        .style(Style::default().fg(Color::Gray).bg(Color::Rgb(38, 38, 38)))
        .alignment(Alignment::Center);

    let instructions_area =
        ratatui::layout::Rect::new(inner.x, inner.y + list_height, inner.width, 2);

    f.render_widget(instructions, instructions_area);
}
