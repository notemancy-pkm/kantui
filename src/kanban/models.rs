// Define a structure for a task
pub struct Task {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<u8>,
}

// Define a structure for a column
pub struct Column {
    pub title: String,
    pub tasks: Vec<Task>,
    pub selected_task: Option<usize>, // Will only matter for the active column
}

// Define input modes
#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    AddingColumn,
    AddingTask,
    MoveMode,
    ConfirmDeleteColumn,
    BoardSelection, // New mode for board selection popup
    AddingBoard,    // New mode for creating a new board
    ColumnSelectionMode,
    JumpToColumnMode,
    JumpToTaskMode,
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
    // Storage fields
    pub file_path: Option<String>,
    // Board selection fields
    pub available_boards: Vec<String>,
    pub selected_board_index: Option<usize>,
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
                        priority: Some(5),
                    },
                    Task {
                        title: "Add task functionality".to_string(),
                        description: None,
                        priority: Some(5),
                    },
                ],
                selected_task: Some(0), // Select the first task by default
            }],
            active_column: 0,
            start_index: 0,
            scroll_offset: 0,
            input_mode: InputMode::BoardSelection, // Start in board selection mode
            input_text: String::new(),
            file_path: None,
            available_boards: Vec::new(),
            selected_board_index: Some(0), // Select first board by default
        };

        // Initialize board selection
        app.scan_available_boards();

        app
    }

    // Scan for available board files in KANBAN_DIR
    pub fn scan_available_boards(&mut self) -> Result<(), std::io::Error> {
        self.available_boards.clear();

        // Check for KANBAN_DIR environment variable
        let kanban_dir = match std::env::var("KANBAN_DIR") {
            Ok(dir) => dir,
            Err(_) => {
                // If KANBAN_DIR is not set, return without scanning
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "KANBAN_DIR environment variable not set",
                ));
            }
        };

        // Create directory if it doesn't exist
        let dir_path = std::path::Path::new(&kanban_dir);
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)?;
        }

        // Scan directory for .txt files
        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            // Check if it's a file with .txt extension
            if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                // Extract board name from filename
                if let Some(file_name) = path.file_stem() {
                    if let Some(name) = file_name.to_str() {
                        // Convert from snake_case to a readable format
                        let display_name = name.replace("_", " ").to_string();
                        self.available_boards.push(display_name);
                    }
                }
            }
        }

        // Sort boards alphabetically
        self.available_boards.sort();

        // Add a "Create New Board" option at the end
        self.available_boards.push("[Create New Board]".to_string());

        // Reset selected index to 0 if there are boards
        if !self.available_boards.is_empty() {
            self.selected_board_index = Some(0);
        } else {
            self.selected_board_index = None;
        }

        Ok(())
    }

    // Create and load a new board
    pub fn create_new_board(&mut self, title: &str) -> Result<(), std::io::Error> {
        self.title = title.to_string();

        // Reset to default columns
        self.columns = vec![Column {
            title: "To Do".to_string(),
            tasks: Vec::new(),
            selected_task: None,
        }];

        self.active_column = 0;

        // Create filename from board title
        let kanban_dir = std::env::var("KANBAN_DIR").map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "KANBAN_DIR environment variable not set",
            )
        })?;

        let dir_path = std::path::Path::new(&kanban_dir);
        let file_name = format!("{}.txt", title.replace(" ", "_").to_lowercase());
        let file_path = dir_path.join(file_name);

        // Store the full file path
        self.file_path = Some(file_path.to_string_lossy().to_string());

        // Save the new board
        self.save_board()?;

        Ok(())
    }

    // Load selected board
    pub fn load_selected_board(&mut self) -> Result<(), std::io::Error> {
        if let Some(index) = self.selected_board_index {
            // Check if it's the "Create New Board" option
            if index == self.available_boards.len() - 1 {
                // Switch to board creation mode
                self.input_mode = InputMode::AddingBoard;
                self.input_text.clear();
                return Ok(());
            }

            // Get selected board name
            if let Some(board_name) = self.available_boards.get(index) {
                // Convert display name back to filename
                let file_name = format!("{}.txt", board_name.replace(" ", "_").to_lowercase());

                // Get KANBAN_DIR
                let kanban_dir = std::env::var("KANBAN_DIR").map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "KANBAN_DIR environment variable not set",
                    )
                })?;

                let dir_path = std::path::Path::new(&kanban_dir);
                let file_path = dir_path.join(file_name);

                // Store the full file path
                self.file_path = Some(file_path.to_string_lossy().to_string());

                // Update the title
                self.title = board_name.clone();

                // Load the board
                self.load_board()?;

                // Switch to normal mode
                self.input_mode = InputMode::Normal;
            }
        }

        Ok(())
    }

    // Rest of the App implementation...
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
                priority: Some(5),
            };

            column.tasks.push(new_task);

            // Select the newly added task in the active column
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

    pub fn select_prev_column(&mut self) {
        if self.active_column > 0 {
            self.active_column -= 1;

            // Clear selection in all non-active columns
            for (i, column) in self.columns.iter_mut().enumerate() {
                if i != self.active_column {
                    column.selected_task = None;
                }
            }
        }
    }

    pub fn select_next_column(&mut self) {
        if self.active_column < self.columns.len().saturating_sub(1) {
            self.active_column += 1;

            // Clear selection in all non-active columns
            for (i, column) in self.columns.iter_mut().enumerate() {
                if i != self.active_column {
                    column.selected_task = None;
                }
            }
        }
    }

    // Board selection navigation
    pub fn select_prev_board(&mut self) {
        if let Some(index) = self.selected_board_index {
            if index > 0 {
                self.selected_board_index = Some(index - 1);
            }
        }
    }

    pub fn select_next_board(&mut self) {
        if let Some(index) = self.selected_board_index {
            if index < self.available_boards.len() - 1 {
                self.selected_board_index = Some(index + 1);
            }
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

            // Clear selection in all non-active columns
            for (i, column) in self.columns.iter_mut().enumerate() {
                if i != self.active_column {
                    column.selected_task = None;
                }
            }

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

                        // Only update target column selection if we're making it the active column
                        if self.active_column == target_column_idx {
                            target_column.selected_task = Some(target_column.tasks.len() - 1);
                        }

                        // Save changes
                        let _ = self.save_board();
                    }
                }
            }
        }
    }
    // Get all possible jump labels
    pub fn get_jump_labels(&self) -> Vec<char> {
        // Use letters from a to z, excluding ambiguous ones
        // This gives us 24 distinct labels (excluding 'l' and 'o' which can be confused)
        let mut labels: Vec<char> = "abcdefghijkmnpqrstuvwxyz".chars().collect();

        // If we need more labels, add capital letters
        if self.total_task_count() > labels.len() {
            let caps: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
            labels.extend(caps);
        }

        labels
    }

    // Get the jump label for a specific task
    pub fn get_jump_label_for_task(&self, col_idx: usize, task_idx: usize) -> Option<char> {
        let labels = self.get_jump_labels();
        let mut global_task_idx = 0;

        // Count tasks until we reach the specified task
        for (c_idx, col) in self.columns.iter().enumerate() {
            for (t_idx, _) in col.tasks.iter().enumerate() {
                if c_idx == col_idx && t_idx == task_idx {
                    // Found our task, return its label if available
                    return if global_task_idx < labels.len() {
                        Some(labels[global_task_idx])
                    } else {
                        None
                    };
                }
                global_task_idx += 1;
            }
        }

        None
    }

    // Find a task by its jump label
    pub fn get_task_by_jump_label(&self, label: char) -> Option<(usize, usize)> {
        let labels = self.get_jump_labels();
        let label_idx = labels.iter().position(|&c| c == label)?;

        let mut global_task_idx = 0;

        // Iterate through all tasks to find the one with matching label index
        for (col_idx, col) in self.columns.iter().enumerate() {
            for (task_idx, _) in col.tasks.iter().enumerate() {
                if global_task_idx == label_idx {
                    return Some((col_idx, task_idx));
                }
                global_task_idx += 1;
            }
        }

        None
    }

    // Count total tasks across all columns
    pub fn total_task_count(&self) -> usize {
        self.columns.iter().map(|col| col.tasks.len()).sum()
    }

    // Jump to a specific task
    pub fn jump_to_task(&mut self, column_idx: usize, task_idx: usize) {
        if column_idx < self.columns.len() {
            // First, switch to the column
            self.active_column = column_idx;

            // Clear selection in all columns
            for (i, column) in self.columns.iter_mut().enumerate() {
                if i != self.active_column {
                    column.selected_task = None;
                }
            }

            // Then select the task if it exists
            if let Some(column) = self.columns.get_mut(column_idx) {
                if task_idx < column.tasks.len() {
                    column.selected_task = Some(task_idx);
                }
            }

            // Exit jump mode
            self.input_mode = InputMode::Normal;

            // Save changes to file
            let _ = self.save_board();
        }
    }
}
