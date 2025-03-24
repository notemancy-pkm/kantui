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
    AddingTask,
    MoveMode,
    ConfirmDeleteColumn,
}

// Define the application structure with added storage fields
pub struct App {
    pub title: String,
    pub columns: Vec<Column>,
    pub active_column: usize,
    pub scroll_offset: usize,
    pub input_mode: InputMode,
    pub input_text: String,
    pub start_index: usize,
    // New field for file storage
    pub file_path: Option<String>,
}

impl App {
    pub fn new(title: &str) -> App {
        let mut app = App {
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
            file_path: None,
        };

        // Try to initialize storage (might fail if KANBAN_DIR not set)
        let _ = app.initialize_storage();

        app
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

        // Save changes to file
        let _ = self.save_board();

        // Exit input mode
        self.input_mode = InputMode::Normal;
        self.input_text.clear();
    }

    pub fn add_task(&mut self, title: &str) {
        if let Some(column) = self.columns.get_mut(self.active_column) {
            let new_task = Task {
                title: title.to_string(),
                description: None,
            };

            column.tasks.push(new_task);

            // Select the newly added task
            column.selected_task = Some(column.tasks.len() - 1);

            // Save changes to file
            let _ = self.save_board();

            // Exit input mode
            self.input_mode = InputMode::Normal;
            self.input_text.clear();
        }
    }

    pub fn delete_current_task(&mut self) {
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if let Some(task_idx) = column.selected_task {
                if task_idx < column.tasks.len() {
                    // Remove the task
                    column.tasks.remove(task_idx);

                    // Adjust the selection
                    if column.tasks.is_empty() {
                        column.selected_task = None;
                    } else if task_idx >= column.tasks.len() {
                        // If we removed the last task, select the new last task
                        column.selected_task = Some(column.tasks.len() - 1);
                    }

                    // Save changes to file
                    let _ = self.save_board();
                }
            }
        }
    }

    pub fn delete_current_column(&mut self) {
        if self.columns.is_empty() {
            return;
        }
        // Remove the active column.
        self.columns.remove(self.active_column);
        // Adjust active_column if needed.
        if self.active_column >= self.columns.len() && !self.columns.is_empty() {
            self.active_column = self.columns.len() - 1;
        }

        // Save changes to file
        let _ = self.save_board();
    }

    // pub fn scroll_left(&mut self) {
    //     if self.scroll_offset > 0 {
    //         self.scroll_offset -= 1;
    //     }
    // }

    // pub fn scroll_right(&mut self) {
    //     if self.scroll_offset < self.columns.len().saturating_sub(1) {
    //         self.scroll_offset += 1;
    //     }
    // }

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

    pub fn jump_to_column(&mut self, index: usize) {
        // Only allow jumping to columns that exist
        // Limit to 0-9 (columns 1-10, with 0 mapped to the first column)
        let target = if index == 0 { 0 } else { index - 1 };

        if target < self.columns.len() {
            self.active_column = target;
            // Exit move mode after jumping
            self.input_mode = InputMode::Normal;

            // Save changes to file
            let _ = self.save_board();
        }
    }

    pub fn move_task_to_column(&mut self, target_column_idx: usize) {
        // Validate target column index
        if target_column_idx >= self.columns.len() || target_column_idx == self.active_column {
            return;
        }

        // Get source column and check if a task is selected
        if let Some(src_column) = self.columns.get_mut(self.active_column) {
            if let Some(task_idx) = src_column.selected_task {
                if task_idx < src_column.tasks.len() {
                    // Remove task from source column
                    let task = src_column.tasks.remove(task_idx);

                    // Update selection in source column
                    if src_column.tasks.is_empty() {
                        src_column.selected_task = None;
                    } else if task_idx >= src_column.tasks.len() {
                        src_column.selected_task = Some(src_column.tasks.len() - 1);
                    }

                    // Add task to target column
                    if let Some(target_column) = self.columns.get_mut(target_column_idx) {
                        target_column.tasks.push(task);
                        target_column.selected_task = Some(target_column.tasks.len() - 1);

                        // Save changes
                        let _ = self.save_board();
                    }
                }
            }
        }
    }
}
