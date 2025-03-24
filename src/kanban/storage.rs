use crate::kanban::models::{App, Column, InputMode, Task};
use crate::ops::crud;
use chrono::Local;
use std::fs;
use std::io;
use std::path::Path;

/// Helper functions to convert between frontend and backend models
impl App {
    /// Initialize the app with KANBAN_DIR environment check
    pub fn initialize_storage(&mut self) -> Result<(), io::Error> {
        // Check for KANBAN_DIR environment variable
        let kanban_dir = match std::env::var("KANBAN_DIR") {
            Ok(dir) => dir,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "KANBAN_DIR environment variable not set",
                ));
            }
        };

        // Create directory if it doesn't exist
        let dir_path = Path::new(&kanban_dir);
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }

        // Create filename from board title
        let file_name = format!("{}.txt", self.title.replace(" ", "_").to_lowercase());
        let file_path = dir_path.join(file_name);

        // Store the full file path
        self.file_path = Some(file_path.to_string_lossy().to_string());

        // If file exists, load it; otherwise, save current state
        if file_path.exists() {
            self.load_board()?;
        } else {
            self.save_board()?;
        }

        Ok(())
    }

    /// Load board from file
    pub fn load_board(&mut self) -> Result<(), io::Error> {
        if let Some(path) = &self.file_path {
            let backend_board = crud::read_board(path)?;
            self.update_from_backend_board(backend_board);
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No file path set"))
        }
    }

    /// Save board to file
    pub fn save_board(&self) -> Result<(), io::Error> {
        if let Some(path) = &self.file_path {
            let backend_board = self.to_backend_board();
            crud::update_board(path, &backend_board)?;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No file path set"))
        }
    }

    fn to_backend_board(&self) -> crud::Board {
        let mut board = crud::Board::new(
            &self.title,
            &Local::now().format("%Y-%m-%d").to_string(),
            "TUI Kanban Board",
        );

        // Add all columns and their tasks
        for column in &self.columns {
            board.add_column(&column.title);

            for task in &column.tasks {
                // Use the priority value if available, otherwise use default values
                let priority_impact = task.priority.unwrap_or(5);

                let backend_task = crud::Task {
                    id: task_to_id(task),
                    title: task.title.clone(),
                    priority: Some(crud::Priority {
                        impact: priority_impact, // Set impact directly from the task priority
                        urgency: 5,              // Default urgency
                        effort: 3,               // Default effort
                    }),
                    tags: Vec::new(),
                    created: Some(Local::now().format("%Y-%m-%d").to_string()),
                };

                let _ = board.add_task(&column.title, backend_task);
            }
        }

        board
    }

    /// Update frontend App from backend Board
    /// Update frontend App from backend Board
    fn update_from_backend_board(&mut self, board: crud::Board) {
        // Store original active column name to restore selection
        let active_column_name = self
            .columns
            .get(self.active_column)
            .map(|col| col.title.clone());

        // Clear existing columns
        self.columns.clear();

        // Add columns from backend board
        for backend_column in &board.columns {
            let mut column = Column {
                title: backend_column.name.clone(),
                tasks: Vec::new(),
                selected_task: None,
            };

            // Add tasks to this column
            for backend_task in &backend_column.tasks {
                // Extract priority from the backend task
                let priority = if let Some(ref prio) = backend_task.priority {
                    Some(prio.impact) // Use impact as the frontend priority value
                } else {
                    None
                };

                let task = Task {
                    title: backend_task.title.clone(),
                    description: None,
                    priority,
                };

                column.tasks.push(task);
            }

            self.columns.push(column);
        }

        // Restore active column if possible
        if let Some(name) = active_column_name {
            for (i, column) in self.columns.iter().enumerate() {
                if column.title == name {
                    self.active_column = i;
                    break;
                }
            }
        }

        // Ensure active_column is valid
        if self.columns.is_empty() {
            self.active_column = 0;
        } else if self.active_column >= self.columns.len() {
            self.active_column = self.columns.len() - 1;
        }

        // Set selection only for the active column
        for (i, column) in self.columns.iter_mut().enumerate() {
            if i == self.active_column && !column.tasks.is_empty() {
                column.selected_task = Some(0);
            } else {
                column.selected_task = None;
            }
        }
    }
}

// Helper function to generate a unique ID for a task based on its content
fn task_to_id(task: &Task) -> usize {
    // Simple hash of the task title
    let mut id: usize = 0;
    for b in task.title.bytes() {
        id = id.wrapping_mul(31).wrapping_add(b as usize);
    }
    id
}
