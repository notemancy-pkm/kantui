pub mod input_handler;
pub mod popups;
pub mod render;
pub mod task_formatter;

// Re-export functions for external use:
pub use input_handler::run_app;
pub use render::draw_ui;
