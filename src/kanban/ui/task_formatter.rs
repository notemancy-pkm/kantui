use crate::kanban::models::Task;
use ratatui::style::Modifier;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
};

/// Calculates a priority color based on the priority value
fn get_priority_color(priority: Option<u8>) -> Color {
    match priority {
        Some(p) if p >= 8 => Color::Red,    // High priority
        Some(p) if p >= 5 => Color::Yellow, // Medium priority
        Some(p) if p >= 3 => Color::Green,  // Normal priority
        Some(_) => Color::Blue,             // Low priority
        None => Color::DarkGray,            // No priority
    }
}

/// Formats task text with wrapping given a maximum width.
/// This function returns a `Text` object that can be rendered in the UI.
/// If a jump_label is provided, it will be displayed next to the task.
pub fn format_task_with_wrapping(
    task: &Task,
    max_width: u16,
    jump_label: Option<char>,
    show_jump_labels: bool,
) -> Text<'static> {
    let task_text = &task.title;
    let indent = "   ";
    let horizontal_padding: usize = 2;
    let effective_width = max_width as usize - (horizontal_padding * 2);
    let max_chars_first_line = effective_width;
    let max_chars_other_lines = effective_width.saturating_sub(indent.len());

    let mut lines = Vec::new();

    // Add an initial padding line with the priority dot
    let priority_color = get_priority_color(task.priority);
    let mut first_padding_line = vec![
        Span::raw(" ".repeat(horizontal_padding)),
        Span::raw(" ".repeat(effective_width - 1)),
        Span::styled("●", Style::default().fg(priority_color)),
        Span::raw(" ".repeat(horizontal_padding)),
    ];
    lines.push(Line::from(first_padding_line));

    // Format the first line with jump label if provided
    let first_line_text = if task_text.len() > max_chars_first_line {
        task_text[..max_chars_first_line].to_string()
    } else {
        task_text.to_string()
    };

    // Calculate space needed for jump label display
    let jump_label_width = if show_jump_labels { 3 } else { 0 }; // "[a]" takes 3 chars
    let available_text_width = effective_width.saturating_sub(jump_label_width);

    // Adjust text to fit within available width after jump label
    let adjusted_text = if first_line_text.len() > available_text_width && available_text_width > 0
    {
        first_line_text[..available_text_width].to_string()
    } else {
        first_line_text.clone()
    };

    let text_padding = effective_width.saturating_sub(adjusted_text.len() + jump_label_width);

    // Create the first line with optional jump label
    let mut first_line_spans = vec![Span::raw(" ".repeat(horizontal_padding))];

    if show_jump_labels {
        if let Some(label) = jump_label {
            // Add the jump label with magenta color for visibility
            first_line_spans.push(Span::styled(
                format!("[{}]", label),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            // Add placeholder space where the label would be
            first_line_spans.push(Span::raw("   "));
        }
    }

    first_line_spans.push(Span::raw(adjusted_text));
    first_line_spans.push(Span::raw(" ".repeat(text_padding)));
    first_line_spans.push(Span::raw(" ".repeat(horizontal_padding)));

    lines.push(Line::from(first_line_spans));

    // If the text is too long, add additional wrapped lines.
    if task_text.len() > max_chars_first_line {
        let remaining_text = task_text[max_chars_first_line..].to_string();
        let mut position = 0;
        while position < remaining_text.len() {
            let end_pos = std::cmp::min(position + max_chars_other_lines, remaining_text.len());
            let line_text = remaining_text[position..end_pos].to_string();
            let line_padding = effective_width.saturating_sub(line_text.len() + indent.len());
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(horizontal_padding)),
                Span::raw(indent),
                Span::raw(line_text),
                Span::raw(" ".repeat(line_padding)),
                Span::raw(" ".repeat(horizontal_padding)),
            ]));
            position = end_pos;
        }
    }

    // Add a final padding line.
    lines.push(Line::from(vec![Span::raw("")]));

    Text::from(lines)
}
