use crate::kanban::models::{App, InputMode};
use ratatui::{
    Frame,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use std::io;

// Define constants for UI elements
const COLUMN_WIDTH: u16 = 50;
const COLUMN_MARGIN: u16 = 2;

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create the title
    let title = Paragraph::new(app.title.clone())
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default());

    // Calculate layout for title and columns area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    f.render_widget(title, chunks[0]);

    // Calculate how many columns we can show at once
    let available_width = chunks[1].width;
    let column_with_margin = COLUMN_WIDTH + (COLUMN_MARGIN * 2);
    let max_visible_columns = (available_width / column_with_margin).max(1) as usize;

    // Determine the starting column index based on the active column
    let start_idx = if app.columns.len() <= max_visible_columns {
        // If all columns fit, show all from the beginning
        0
    } else if app.active_column >= app.scroll_offset + max_visible_columns {
        // If active column is beyond current view, adjust scroll
        app.active_column + 1 - max_visible_columns
    } else if app.active_column < app.scroll_offset {
        // If active column is before current view, adjust scroll
        app.active_column
    } else {
        // Otherwise, keep current scroll position
        app.scroll_offset
    };

    // Calculate how many columns we can actually show
    let visible_columns = max_visible_columns.min(app.columns.len() - start_idx);

    // Create constraints for the visible columns with margins
    let mut column_constraints = Vec::new();
    for _ in 0..visible_columns {
        // Left margin
        column_constraints.push(Constraint::Length(COLUMN_MARGIN));
        // Column
        column_constraints.push(Constraint::Length(COLUMN_WIDTH));
        // Right margin
        column_constraints.push(Constraint::Length(COLUMN_MARGIN));
    }

    // If we have space left, add it as an extra constraint
    if !column_constraints.is_empty()
        && available_width > column_with_margin * (column_constraints.len() / 3) as u16
    {
        column_constraints.push(Constraint::Min(0));
    }

    let columns_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(column_constraints)
        .split(chunks[1]);

    // Render visible columns
    for (layout_idx, column_idx) in (start_idx..app.columns.len())
        .enumerate()
        .take(visible_columns)
    {
        let column = &app.columns[column_idx];

        // Calculate the actual column area (skip margin constraints)
        let column_area = columns_layout[layout_idx * 3 + 1]; // +1 to skip left margin

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

        // Create a custom block with just a top border for the column title
        let title_text = Paragraph::new(title_with_index)
            .alignment(Alignment::Center)
            .style(style);

        // Create a blue horizontal line
        let horizontal_line = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Blue));

        // Layout for title and content within column
        let column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Blue line
                Constraint::Min(1),    // Content
            ])
            .split(column_area);

        f.render_widget(title_text, column_layout[0]);
        f.render_widget(horizontal_line, column_layout[1]);

        let tasks: Vec<ListItem> = column
            .tasks
            .iter()
            .enumerate()
            .flat_map(|(i, task)| {
                let task_style = if column.selected_task == Some(i) {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Add a circle bullet and some padding
                let task_text = format!(" ● {}  ", task.title);

                // Create the task item
                let task_item = ListItem::new(Span::styled(task_text, task_style))
                    .style(Style::default().fg(Color::Reset));

                // Add a blank line item after each task (except the last one)
                if i < column.tasks.len() - 1 {
                    vec![task_item, ListItem::new("")]
                } else {
                    vec![task_item]
                }
            })
            .collect();

        let tasks_list = List::new(tasks);

        f.render_widget(tasks_list, column_layout[2]);
    }

    // Add navigation help at the bottom
    let help_text = match app.input_mode {
        InputMode::Normal => {
            "Use 'h'/'l' to navigate columns | 'j'/'k' to navigate tasks | Ctrl+L to add a column | Ctrl+M for move mode | 'q' to quit"
        }
        InputMode::AddingColumn => "Enter column name | Enter to confirm | Esc to cancel",
        InputMode::MoveMode => "Press 0-9 to jump to that column | Esc to cancel",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    let help_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(size);

    f.render_widget(help, help_layout[1]);

    // Draw popup overlay for adding a column
    if let InputMode::AddingColumn = app.input_mode {
        // Create a centered popup
        let popup_width = 50;
        let popup_height = 3;
        let popup_area = ratatui::layout::Rect::new(
            (size.width.saturating_sub(popup_width)) / 2,
            (size.height.saturating_sub(popup_height)) / 2,
            popup_width.min(size.width),
            popup_height.min(size.height),
        );

        // Create a clear background for the popup
        f.render_widget(Clear, popup_area);

        // Create the popup block
        let popup_block = Block::default()
            .title("New Column")
            .borders(Borders::ALL)
            .style(Style::default());

        f.render_widget(&popup_block, popup_area);

        // Create the input field
        let input_area = popup_block.inner(popup_area);

        let input = Paragraph::new(app.input_text.clone())
            .style(Style::default())
            .block(Block::default());

        f.render_widget(input, input_area);

        // Set cursor to the end of input
        f.set_cursor_position(Position {
            x: input_area.x + app.input_text.len() as u16,
            y: input_area.y,
        });
    }

    if let InputMode::MoveMode = app.input_mode {
        // Create a small indicator at the bottom of the screen
        let mode_indicator = Paragraph::new("MOVE MODE")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);

        let indicator_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1), // Help text
                Constraint::Length(1), // Move mode indicator
            ])
            .split(size);

        f.render_widget(mode_indicator, indicator_layout[2]);

        // Optionally, show column numbers
        // Calculate visible columns (similar to your existing column rendering logic)
        // Then add a number indicator to each column title
        for (layout_idx, column_idx) in (start_idx..app.columns.len())
            .enumerate()
            .take(visible_columns)
        {
            if column_idx < 10 {
                // Only for columns 0-9
                let column_area = columns_layout[layout_idx * 3 + 1];
                let number_indicator = Paragraph::new(format!(
                    "[{}]",
                    if column_idx == 0 { 0 } else { column_idx + 1 }
                ))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);

                // Render just above the column title
                let indicator_area = ratatui::layout::Rect::new(
                    column_area.x,
                    column_area.y.saturating_sub(1),
                    column_area.width,
                    1,
                );

                if indicator_area.y > 0 {
                    f.render_widget(number_indicator, indicator_area);
                }
            }
        }
    }
}

pub fn run_app(
    terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};

    loop {
        // Calculate the max visible columns before drawing
        let backend_size = terminal.size()?;
        // Subtract space for padding, title, help text, etc.
        let available_width = backend_size.width;
        let column_with_margin = COLUMN_WIDTH + (COLUMN_MARGIN * 2);
        let max_visible_columns = (available_width / column_with_margin).max(1) as usize;

        // Update scroll_offset based on active_column and max_visible_columns
        if app.columns.len() <= max_visible_columns {
            // If all columns fit, show all from the beginning
            app.scroll_offset = 0;
        } else if app.active_column >= app.scroll_offset + max_visible_columns {
            // If active column is beyond current view, adjust scroll
            app.scroll_offset = app.active_column + 1 - max_visible_columns;
        } else if app.active_column < app.scroll_offset {
            // If active column is before current view, adjust scroll
            app.scroll_offset = app.active_column;
        }
        // Otherwise keep current scroll_offset

        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                        app.input_mode = InputMode::AddingColumn;
                    }
                    KeyCode::Char('h') => {
                        app.select_prev_column();
                    }
                    KeyCode::Char('k') if key.modifiers == KeyModifiers::CONTROL => {
                        // Enter move mode with Ctrl+M
                        app.input_mode = InputMode::MoveMode;
                    }
                    KeyCode::Char('l') => {
                        app.select_next_column();
                    }
                    KeyCode::Char('j') => {
                        app.select_next_task();
                    }
                    KeyCode::Char('k') => {
                        app.select_prev_task();
                    }
                    _ => {}
                },
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
                    KeyCode::Char(c) => {
                        app.input_text.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_text.pop();
                    }
                    _ => {}
                },
                InputMode::MoveMode => match key.code {
                    KeyCode::Esc => {
                        // Exit move mode with Escape
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) if c >= '0' && c <= '9' => {
                        // Jump to column by number (0-9)
                        let index = c.to_digit(10).unwrap() as usize;
                        app.jump_to_column(index);
                    }
                    _ => {}
                },
            }
        }
    }
}
