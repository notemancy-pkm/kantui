use crate::kanban::models::{App, InputMode};
use crate::kanban::ui::popups;
use crate::kanban::ui::task_formatter::format_task_with_wrapping;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

const COLUMN_WIDTH: u16 = 50;
const COLUMN_MARGIN: u16 = 2;

/// Draws the overall UI including the title, columns, tasks and any popups.
pub fn draw_ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Set the background color for the entire app
    // let background = Block::default()
    //     .style(Style::default().bg(Color::Rgb(22, 22, 22))) // #161616
    //     .borders(Borders::NONE);
    // f.render_widget(background, size);

    // Clear the terminal with our background color
    f.render_widget(Clear, size); // First clear any existing content
    let background = Block::default()
        .style(Style::default().bg(Color::Rgb(22, 22, 22))) // #161616
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Check if we need to show the board selection popup
    match app.input_mode {
        InputMode::BoardSelection => {
            popups::draw_board_selection(f, app, size);
            return;
        }
        InputMode::AddingBoard => {
            popups::draw_new_board_popup(f, app, size);
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
        InputMode::RenamingColumn => "Edit column name | Enter to confirm | Esc to cancel",
        InputMode::RenamingTask => "Edit task name | Enter to confirm | Esc to cancel",
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

/// Helper function to draw the new board creation popup

fn draw_popup(f: &mut Frame, app: &App, size: ratatui::layout::Rect) {
    popups::draw_popup(f, app, size);
}
