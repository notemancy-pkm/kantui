use crate::kanban::models::{App, InputMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

/// Draw a simple input popup with a title and input field
pub fn draw_input_popup(
    f: &mut Frame,
    app: &App,
    size: Rect,
    title: &str,
    width: u16,
    height: u16,
) {
    let popup_area = Rect::new(
        (size.width.saturating_sub(width)) / 2,
        (size.height.saturating_sub(height)) / 2,
        width.min(size.width),
        height.min(size.height),
    );

    f.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title(title)
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

/// Draw the confirmation popup for deleting a column
pub fn draw_confirm_delete_column(f: &mut Frame, app: &App, size: Rect) {
    let popup_width = 50;
    let popup_height = 3;

    let popup_area = Rect::new(
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

/// Draw the column selection popup for moving tasks
pub fn draw_column_selection_popup(f: &mut Frame, app: &App, size: Rect) {
    let popup_width = 50;
    let popup_height = std::cmp::min(app.columns.len() as u16 + 4, 15); // Max height of 15

    let popup_area = Rect::new(
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

    let list_area = Rect::new(inner.x, inner.y, inner.width, list_height);

    f.render_widget(columns_list, list_area);

    // Instructions
    let instructions = Paragraph::new("Press a number to move task to that column, Esc to cancel")
        .style(Style::default().fg(Color::Gray).bg(Color::Rgb(38, 38, 38)))
        .alignment(Alignment::Center);

    let instructions_area = Rect::new(inner.x, inner.y + list_height, inner.width, 2);

    f.render_widget(instructions, instructions_area);
}

/// Draw the jump column popup
pub fn draw_jump_column_popup(f: &mut Frame, app: &App, size: Rect) {
    let popup_width = 50;
    let popup_height = std::cmp::min(app.columns.len() as u16 + 4, 15); // Max height of 15

    let popup_area = Rect::new(
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

    let list_area = Rect::new(inner.x, inner.y, inner.width, list_height);

    f.render_widget(columns_list, list_area);

    // Instructions
    let instructions = Paragraph::new("Press a number to jump to that column, Esc to cancel")
        .style(Style::default().fg(Color::Gray).bg(Color::Rgb(38, 38, 38)))
        .alignment(Alignment::Center);

    let instructions_area = Rect::new(inner.x, inner.y + list_height, inner.width, 2);

    f.render_widget(instructions, instructions_area);
}

/// Draw the board selection popup
pub fn draw_board_selection(f: &mut Frame, app: &App, size: Rect) {
    // Clear the screen with background color
    let background = Block::default()
        .style(Style::default().bg(Color::Rgb(22, 22, 22))) // #161616
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Create a centered popup
    let popup_width = 60;
    let popup_height = std::cmp::min(20, app.available_boards.len() as u16 + 6);

    let popup_area = Rect::new(
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

/// Draw the new board creation popup
pub fn draw_new_board_popup(f: &mut Frame, app: &App, size: Rect) {
    // Set background color
    let background = Block::default()
        .style(Style::default().bg(Color::Rgb(22, 22, 22))) // #161616
        .borders(Borders::NONE);
    f.render_widget(background, size);

    let popup_width = 60;
    let popup_height = 5;

    let popup_area = Rect::new(
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

/// Main function to draw popups based on the input mode
pub fn draw_popup(f: &mut Frame, app: &App, size: Rect) {
    match app.input_mode {
        InputMode::BoardSelection => {
            // Already handled separately in the main draw_ui function
        }
        InputMode::AddingBoard => {
            // Already handled separately in the main draw_ui function
        }
        InputMode::ColumnSelectionMode => {
            draw_column_selection_popup(f, app, size);
        }
        InputMode::AddingColumn => {
            draw_input_popup(f, app, size, "New Column", 70, 5);
        }
        InputMode::AddingTask => {
            draw_input_popup(f, app, size, "New Task", 70, 5);
        }
        InputMode::RenamingColumn => {
            draw_input_popup(f, app, size, "Rename Column", 70, 5);
        }
        InputMode::RenamingTask => {
            draw_input_popup(f, app, size, "Rename Task", 70, 5);
        }
        InputMode::ConfirmDeleteColumn => {
            draw_confirm_delete_column(f, app, size);
        }
        InputMode::JumpToColumnMode => {
            draw_jump_column_popup(f, app, size);
        }
        InputMode::Normal | InputMode::MoveMode | InputMode::JumpToTaskMode => {
            // No popups for these modes
        }
    }
}
