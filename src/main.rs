use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io;

// Define a structure for a task
struct Task {
    title: String,
    description: Option<String>,
}

// Define a structure for a column
struct Column {
    title: String,
    tasks: Vec<Task>,
}

// Define the application structure
struct App {
    title: String,
    columns: Vec<Column>,
    active_column: usize,
    scroll_offset: usize,
}

impl App {
    fn new(title: &str) -> App {
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
            }],
            active_column: 0,
            scroll_offset: 0,
        }
    }

    fn add_column(&mut self, title: &str) {
        self.columns.push(Column {
            title: title.to_string(),
            tasks: Vec::new(),
        });
    }

    fn scroll_left(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_right(&mut self) {
        if self.scroll_offset < self.columns.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    fn select_prev_column(&mut self) {
        if self.active_column > 0 {
            self.active_column -= 1;
            // Adjust scroll if needed
            if self.active_column < self.scroll_offset {
                self.scroll_offset = self.active_column;
            }
        }
    }

    fn select_next_column(&mut self) {
        if self.active_column < self.columns.len().saturating_sub(1) {
            self.active_column += 1;
            // Adjust scroll if needed - reduced columns shown due to margins
            if self.active_column >= self.scroll_offset + 2 {
                // Now showing 2 columns at a time with margins
                self.scroll_offset = self.active_column.saturating_sub(1);
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new("My Kanban Board");
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    return Ok(());
                }
                KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                    app.add_column("New Column");
                }
                KeyCode::Char('h') => {
                    app.select_prev_column();
                }
                KeyCode::Char('l') => {
                    app.select_next_column();
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.area();

    // Create the title
    let title = Paragraph::new(app.title.clone())
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));

    // Calculate layout for title and columns area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    f.render_widget(title, chunks[0]);

    // Define a fixed column width (50 characters)
    const COLUMN_WIDTH: u16 = 50;
    // Define margin between columns
    const COLUMN_MARGIN: u16 = 2;

    // Calculate how many columns we can show at once
    let available_width = chunks[1].width;
    let column_with_margin = COLUMN_WIDTH + (COLUMN_MARGIN * 2);
    let visible_columns = (available_width / column_with_margin).max(1) as usize;

    // Create constraints for the visible columns with margins
    let mut column_constraints = Vec::new();
    for _ in 0..visible_columns.min(app.columns.len().saturating_sub(app.scroll_offset)) {
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
    for (layout_idx, column_idx) in (app.scroll_offset..app.columns.len())
        .enumerate()
        .take(visible_columns)
    {
        let column = &app.columns[column_idx];

        // Calculate the actual column area (skip margin constraints)
        let column_area = columns_layout[layout_idx * 3 + 1]; // +1 to skip left margin
        let border_style = if column_idx == app.active_column {
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
            .style(if column_idx == app.active_column {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

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
            .map(|task| ListItem::new(Span::raw(&task.title)))
            .collect();

        let tasks_list =
            List::new(tasks).highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(tasks_list, column_layout[2]);
    }

    // Add navigation help at the bottom
    let help_text = "Use 'h'/'l' to navigate columns | Ctrl+L to add a column | 'q' to quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    let help_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(size);

    f.render_widget(help, help_layout[1]);
}
