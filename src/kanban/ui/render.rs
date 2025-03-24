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
                let formatted_task = format_task_with_wrapping(&task.title, column_area.width);
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
            "Use 'h'/'l' to navigate columns | 'j'/'k' to navigate tasks | Ctrl+L to add column | Ctrl+T to add task | Ctrl+D to delete task | Ctrl+M for move mode | 'q' to quit"
        }
        InputMode::AddingColumn => "Enter column name | Enter to confirm | Esc to cancel",
        InputMode::AddingTask => "Enter task name | Enter to confirm | Esc to cancel",
        InputMode::MoveMode => "Press 0-9 to jump to that column | Esc to cancel",
        InputMode::ConfirmDeleteColumn => "Press y to delete | n to cancel",
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    let help_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(size);
    f.render_widget(help, help_layout[1]);

    // Draw any popups.
    draw_popup(f, app, size);
}

/// Helper function to draw popups based on the input mode.
fn draw_popup(f: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    match app.input_mode {
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
                .style(Style::default());
            f.render_widget(&popup_block, popup_area);
            let input_area = popup_block.inner(popup_area);
            let input = Paragraph::new(app.input_text.clone())
                .style(Style::default())
                .block(Block::default())
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
                .style(Style::default());
            f.render_widget(&popup_block, popup_area);
            let input_area = popup_block.inner(popup_area);
            let input = Paragraph::new(app.input_text.clone())
                .style(Style::default())
                .block(Block::default())
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
                .style(Style::default());
            f.render_widget(&popup_block, popup_area);
            let inner = popup_block.inner(popup_area);
            let column_name = app
                .columns
                .get(app.active_column)
                .map(|col| col.title.as_str())
                .unwrap_or("");
            let text = Paragraph::new(format!("Delete column '{}' ? (y/n)", column_name))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            f.render_widget(text, inner);
        }
        _ => {}
    }
}
