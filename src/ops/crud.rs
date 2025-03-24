use std::fs;
use std::io::{self, BufRead, BufReader, Write};

/// Represents the priority breakdown: Impact, Urgency, and Effort (each scored 0–10).
#[derive(Debug, Clone, PartialEq)]
pub struct Priority {
    pub impact: u8,
    pub urgency: u8,
    pub effort: u8,
}

impl Priority {
    /// Computes the overall priority as (impact * urgency) / effort.
    /// Returns None if effort is 0.
    pub fn computed(&self) -> Option<f32> {
        if self.effort == 0 {
            None
        } else {
            let base_score = (self.impact as f32 + self.urgency as f32) / self.effort as f32;
            let normalized_score = (base_score - 0.2) / 19.8;
            Some(1.0 + 9.0 * normalized_score)
        }
    }
}

/// A single task on the board.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: usize,
    pub title: String,
    /// Now the priority is represented by a breakdown of impact, urgency, and effort.
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub created: Option<String>,
}

/// A column in the Kanban board (e.g., "To Do", "In Progress", "Done").
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub tasks: Vec<Task>,
}

/// A Kanban board with metadata and a set of columns.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub name: String,
    pub date: String,
    pub description: String,
    pub columns: Vec<Column>,
}

impl Board {
    /// Creates a new board with no columns.
    pub fn new(name: &str, date: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            date: date.to_string(),
            description: description.to_string(),
            columns: Vec::new(),
        }
    }

    /// Adds a new column to the board.
    pub fn add_column(&mut self, column_name: &str) {
        self.columns.push(Column {
            name: column_name.to_string(),
            tasks: Vec::new(),
        });
    }

    /// Adds a task to the specified column.
    pub fn add_task(&mut self, column_name: &str, task: Task) -> Result<(), String> {
        if let Some(column) = self.columns.iter_mut().find(|c| c.name == column_name) {
            column.tasks.push(task);
            Ok(())
        } else {
            Err(format!("Column '{}' not found", column_name))
        }
    }

    /// Updates an existing task identified by task_id in the specified column.
    pub fn update_task(
        &mut self,
        column_name: &str,
        task_id: usize,
        updated_task: Task,
    ) -> Result<(), String> {
        if let Some(column) = self.columns.iter_mut().find(|c| c.name == column_name) {
            if let Some(task) = column.tasks.iter_mut().find(|t| t.id == task_id) {
                *task = updated_task;
                Ok(())
            } else {
                Err(format!(
                    "Task with id {} not found in column '{}'",
                    task_id, column_name
                ))
            }
        } else {
            Err(format!("Column '{}' not found", column_name))
        }
    }

    /// Deletes a task identified by task_id from the specified column.
    pub fn delete_task(&mut self, column_name: &str, task_id: usize) -> Result<(), String> {
        if let Some(column) = self.columns.iter_mut().find(|c| c.name == column_name) {
            let orig_len = column.tasks.len();
            column.tasks.retain(|t| t.id != task_id);
            if column.tasks.len() < orig_len {
                Ok(())
            } else {
                Err(format!(
                    "Task with id {} not found in column '{}'",
                    task_id, column_name
                ))
            }
        } else {
            Err(format!("Column '{}' not found", column_name))
        }
    }

    /// Saves the board to a plain text file in our TUI Kanban Format.
    ///
    /// Task lines now include the breakdown:
    /// * [ID:<id>] Title | Impact: <impact> | Urgency: <urgency> | Effort: <effort> | Computed: <computed> | Tags: tag1,tag2 | Created: <created>
    pub fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let mut file = fs::File::create(file_path)?;
        writeln!(file, "# TUI Kanban Board: {}", self.name)?;
        writeln!(file, "Date: {}", self.date)?;
        writeln!(file, "Description: {}", self.description)?;
        writeln!(file)?;
        for column in &self.columns {
            writeln!(file, "== {} ==", column.name)?;
            for task in &column.tasks {
                let mut task_line = format!("* [ID:{}] {}", task.id, task.title);
                if let Some(ref prio) = task.priority {
                    task_line.push_str(&format!(" | Impact: {}", prio.impact));
                    task_line.push_str(&format!(" | Urgency: {}", prio.urgency));
                    task_line.push_str(&format!(" | Effort: {}", prio.effort));
                    if let Some(computed) = prio.computed() {
                        task_line.push_str(&format!(" | Computed: {:.2}", computed));
                    }
                }
                if !task.tags.is_empty() {
                    task_line.push_str(&format!(" | Tags: {}", task.tags.join(",")));
                }
                if let Some(ref created) = task.created {
                    task_line.push_str(&format!(" | Created: {}", created));
                }
                writeln!(file, "{}", task_line)?;
            }
            writeln!(file)?;
        }
        Ok(())
    }

    /// Loads a board from a plain text file in our TUI Kanban Format.
    ///
    /// It parses the Impact, Urgency, and Effort values (ignoring any computed value).
    pub fn load_from_file(file_path: &str) -> io::Result<Board> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);

        let mut board = Board {
            name: String::new(),
            date: String::new(),
            description: String::new(),
            columns: Vec::new(),
        };
        let mut current_column: Option<Column> = None;

        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("#") {
                if trimmed.contains("TUI Kanban Board:") {
                    if let Some(idx) = trimmed.find("TUI Kanban Board:") {
                        board.name = trimmed[(idx + "TUI Kanban Board:".len())..]
                            .trim()
                            .to_string();
                    }
                }
            } else if trimmed.starts_with("Date:") {
                board.date = trimmed["Date:".len()..].trim().to_string();
            } else if trimmed.starts_with("Description:") {
                board.description = trimmed["Description:".len()..].trim().to_string();
            } else if trimmed.starts_with("==") && trimmed.ends_with("==") {
                if let Some(col) = current_column.take() {
                    board.columns.push(col);
                }
                let col_name = trimmed.trim_matches('=').trim();
                current_column = Some(Column {
                    name: col_name.to_string(),
                    tasks: Vec::new(),
                });
            } else if trimmed.starts_with("*") {
                let mut parts = trimmed.split('|').map(|s| s.trim());
                let first_part = parts.next().unwrap_or("");
                let id_start = first_part.find("[ID:").map(|i| i + 4).unwrap_or(0);
                let id_end = first_part.find(']').unwrap_or(first_part.len());
                let id_str = &first_part[id_start..id_end];
                let id: usize = id_str.parse().unwrap_or(0);
                let title = first_part[id_end + 1..].trim().to_string();

                let mut impact: Option<u8> = None;
                let mut urgency: Option<u8> = None;
                let mut effort: Option<u8> = None;
                let mut tags = Vec::new();
                let mut created = None;

                for part in parts {
                    if part.starts_with("Impact:") {
                        impact = part["Impact:".len()..].trim().parse().ok();
                    } else if part.starts_with("Urgency:") {
                        urgency = part["Urgency:".len()..].trim().parse().ok();
                    } else if part.starts_with("Effort:") {
                        effort = part["Effort:".len()..].trim().parse().ok();
                    } else if part.starts_with("Tags:") {
                        tags = part["Tags:".len()..]
                            .trim()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if part.starts_with("Created:") {
                        created = Some(part["Created:".len()..].trim().to_string());
                    }
                    // Note: We ignore the "Computed:" field since it’s derived.
                }

                let priority = if let (Some(imp), Some(urg), Some(eff)) = (impact, urgency, effort)
                {
                    Some(Priority {
                        impact: imp,
                        urgency: urg,
                        effort: eff,
                    })
                } else {
                    None
                };

                let task = Task {
                    id,
                    title,
                    priority,
                    tags,
                    created,
                };

                if let Some(col) = current_column.as_mut() {
                    col.tasks.push(task);
                }
            }
        }
        if let Some(col) = current_column.take() {
            board.columns.push(col);
        }
        Ok(board)
    }
}

/// Board-level CRUD functions using file storage.
pub fn create_board(file_path: &str, board: &Board) -> io::Result<()> {
    board.save_to_file(file_path)
}

/// Reads a board from a file.
pub fn read_board(file_path: &str) -> io::Result<Board> {
    Board::load_from_file(file_path)
}

/// Updates a board by writing its current state to the file.
pub fn update_board(file_path: &str, board: &Board) -> io::Result<()> {
    board.save_to_file(file_path)
}

/// Deletes the board file.
pub fn delete_board(file_path: &str) -> io::Result<()> {
    fs::remove_file(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;

    #[test]
    fn test_board_creation_and_add_column() {
        let mut board = Board::new("Test Board", "2025-03-24", "Test description");
        board.add_column("To Do");
        board.add_column("Done");

        assert_eq!(board.columns.len(), 2);
        assert_eq!(board.columns[0].name, "To Do");
        assert_eq!(board.columns[1].name, "Done");
    }

    #[test]
    fn test_add_update_delete_task() {
        let mut board = Board::new("Test Board", "2025-03-24", "Test description");
        board.add_column("To Do");

        let task = Task {
            id: 1,
            title: "Test task".to_string(),
            priority: Some(Priority {
                impact: 8,
                urgency: 7,
                effort: 2,
            }),
            tags: vec!["tag1".to_string()],
            created: Some("2025-03-23".to_string()),
        };

        // Test adding a task.
        board.add_task("To Do", task.clone()).unwrap();
        assert_eq!(board.columns[0].tasks.len(), 1);
        assert_eq!(board.columns[0].tasks[0].id, 1);

        // Test updating the task.
        let updated_task = Task {
            id: 1,
            title: "Updated task".to_string(),
            priority: Some(Priority {
                impact: 9,
                urgency: 8,
                effort: 3,
            }),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            created: Some("2025-03-24".to_string()),
        };
        board.update_task("To Do", 1, updated_task.clone()).unwrap();
        assert_eq!(board.columns[0].tasks[0].title, "Updated task");

        // Test deleting the task.
        board.delete_task("To Do", 1).unwrap();
        assert_eq!(board.columns[0].tasks.len(), 0);
    }

    #[test]
    fn test_priority_computed() {
        let priority = Priority {
            impact: 8,
            urgency: 5,
            effort: 2,
        };
        let computed = priority.computed().unwrap();
        assert_eq!(computed, 20.0); // 8 * 5 / 2 = 20.0

        // Test division by zero returns None.
        let priority_zero = Priority {
            impact: 8,
            urgency: 5,
            effort: 0,
        };
        assert!(priority_zero.computed().is_none());
    }

    #[test]
    fn test_save_and_load_board() {
        // Create a temporary file path in the system's temporary directory.
        let mut temp_path = env::temp_dir();
        temp_path.push("test_board.txt");
        let file_path = temp_path.to_str().unwrap();

        let mut board = Board::new(
            "Test Board",
            "2025-03-24",
            "Test board for saving and loading",
        );
        board.add_column("To Do");
        board.add_column("Done");

        let task1 = Task {
            id: 1,
            title: "Task 1".to_string(),
            priority: Some(Priority {
                impact: 5,
                urgency: 5,
                effort: 5,
            }),
            tags: vec!["test".to_string()],
            created: Some("2025-03-23".to_string()),
        };

        let task2 = Task {
            id: 2,
            title: "Task 2".to_string(),
            priority: None,
            tags: vec![],
            created: None,
        };

        board.add_task("To Do", task1.clone()).unwrap();
        board.add_task("Done", task2.clone()).unwrap();

        // Save board to file.
        board.save_to_file(file_path).unwrap();

        // Load board from file.
        let loaded_board = Board::load_from_file(file_path).unwrap();
        assert_eq!(loaded_board.name, "Test Board");
        assert_eq!(loaded_board.columns.len(), 2);
        assert_eq!(loaded_board.columns[0].name, "To Do");
        assert_eq!(loaded_board.columns[0].tasks.len(), 1);
        assert_eq!(loaded_board.columns[0].tasks[0].title, "Task 1");
        assert_eq!(loaded_board.columns[1].tasks.len(), 1);
        assert_eq!(loaded_board.columns[1].tasks[0].title, "Task 2");

        // Clean up: delete the test file.
        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_board_crud_functions() {
        let mut temp_path = env::temp_dir();
        temp_path.push("test_board_crud.txt");
        let file_path = temp_path.to_str().unwrap();

        let board = Board::new("CRUD Board", "2025-03-24", "Board for CRUD tests");
        // Create board file.
        create_board(file_path, &board).unwrap();

        // Read board file.
        let read_back = read_board(file_path).unwrap();
        assert_eq!(read_back.name, "CRUD Board");

        // Update board (add a column) and write again.
        let mut updated_board = read_back;
        updated_board.add_column("To Do");
        update_board(file_path, &updated_board).unwrap();

        let read_updated = read_board(file_path).unwrap();
        assert_eq!(read_updated.columns.len(), 1);

        // Delete board file.
        delete_board(file_path).unwrap();
        assert!(!Path::new(file_path).exists());
    }
}
