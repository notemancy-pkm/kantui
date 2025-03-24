use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io;

// Define a structure for a task
pub struct Task {
    pub title: String,
    pub description: Option<String>,
}

// Define a structure for a column
pub struct Column {
    pub title: String,
    pub tasks: Vec<Task>,
    pub selected_task: Option<usize>,
}

// Define input modes
pub enum InputMode {
    Normal,
    AddingColumn,
    MoveMode,
}

// Define the application structure
pub struct App {
    pub title: String,
    pub columns: Vec<Column>,
    pub active_column: usize,
    pub scroll_offset: usize,
    pub input_mode: InputMode,
    pub input_text: String,
    pub start_index: usize,
}

impl App {
    pub fn new(title: &str) -> App {
        App {
            title: title.to_string(),
            columns: vec![Column {
                title: "To Do".to_string(),
                tasks: vec![
                    Task {
                        title: "Implement UI".to_string(),
                        description: None,
                    },
                    Task {
                        title: "Add task functionality".to_string(),
                        description: None,
                    },
                ],
                selected_task: Some(0), // Select the first task by default
            }],
            active_column: 0,
            start_index: 0,
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            input_text: String::new(),
        }
    }

    pub fn add_column(&mut self, title: &str) {
        // Ensure the column name is unique
        let mut unique_name = title.to_string();
        let mut counter = 1;

        while self.columns.iter().any(|col| col.title == unique_name) {
            unique_name = format!("{} ({})", title, counter);
            counter += 1;
        }

        self.columns.push(Column {
            title: unique_name,
            tasks: Vec::new(),
            selected_task: None, // No tasks selected in a new empty column
        });

        // Exit input mode
        self.input_mode = InputMode::Normal;
        self.input_text.clear();
    }

    pub fn jump_to_column(&mut self, index: usize) {
        // Only allow jumping to columns that exist
        // Limit to 0-9 (columns 1-10, with 0 mapped to the first column)
        let target = if index == 0 { 0 } else { index - 1 };

        if target < self.columns.len() {
            self.active_column = target;
            // Exit move mode after jumping
            self.input_mode = InputMode::Normal;
        }
    }

    pub fn scroll_left(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_right(&mut self) {
        if self.scroll_offset < self.columns.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn select_prev_column(&mut self) {
        if self.active_column > 0 {
            self.active_column -= 1;
        }
    }

    pub fn select_next_column(&mut self) {
        if self.active_column < self.columns.len().saturating_sub(1) {
            self.active_column += 1;
        }
    }

    // Task navigation methods
    pub fn select_prev_task(&mut self) {
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if column.tasks.is_empty() {
                column.selected_task = None;
                return;
            }

            match column.selected_task {
                Some(current) if current > 0 => {
                    column.selected_task = Some(current - 1);
                }
                None if !column.tasks.is_empty() => {
                    // If no task is selected but there are tasks, select the last one
                    column.selected_task = Some(column.tasks.len() - 1);
                }
                _ => {} // Already at the first task or no tasks
            }
        }
    }

    pub fn select_next_task(&mut self) {
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if column.tasks.is_empty() {
                column.selected_task = None;
                return;
            }

            match column.selected_task {
                Some(current) if current < column.tasks.len() - 1 => {
                    column.selected_task = Some(current + 1);
                }
                None if !column.tasks.is_empty() => {
                    // If no task is selected but there are tasks, select the first one
                    column.selected_task = Some(0);
                }
                _ => {} // Already at the last task or no tasks
            }
        }
    }
}

// Calculate the appropriate starting index for column scrolling
pub fn calculate_start_index(app: &App, max_visible_columns: usize) -> usize {
    if app.columns.len() <= max_visible_columns {
        // If all columns fit, show all from the beginning
        0
    } else if app.active_column >= app.start_index + max_visible_columns {
        // If active column is beyond current view, adjust scroll
        app.active_column + 1 - max_visible_columns
    } else if app.active_column < app.start_index {
        // If active column is before current view, adjust scroll
        app.active_column
    } else {
        // Otherwise, keep current scroll position
        app.start_index
    }
}
