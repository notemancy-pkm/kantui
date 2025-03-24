use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
};

/// Formats task text with wrapping given a maximum width.
/// This function returns a `Text` object that can be rendered in the UI.
pub fn format_task_with_wrapping(task_text: &str, max_width: u16) -> Text<'static> {
    let bullet = "✦ ";
    let indent = "  ";
    let horizontal_padding: usize = 1;
    let bullet_len = bullet.chars().count();
    let effective_width = max_width as usize - (horizontal_padding * 2);
    let max_chars_first_line = effective_width.saturating_sub(bullet_len);
    let max_chars_other_lines = effective_width.saturating_sub(indent.len());

    let mut lines = Vec::new();

    // Add an initial padding line.
    // lines.push(Line::from(vec![Span::raw("")]));

    // Format the first line with a bullet.
    let first_line_text = if task_text.len() > max_chars_first_line {
        task_text[..max_chars_first_line].to_string()
    } else {
        task_text.to_string()
    };

    let first_line_padding = effective_width.saturating_sub(first_line_text.len() + bullet_len);

    lines.push(Line::from(vec![
        Span::raw(" ".repeat(horizontal_padding)),
        Span::styled(bullet, Style::default().fg(Color::Yellow)),
        Span::raw(first_line_text),
        // Span::raw(" ".repeat(first_line_padding)),
        Span::raw(" ".repeat(horizontal_padding)),
    ]));

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
    // lines.push(Line::from(vec![Span::raw("")]));

    Text::from(lines)
}
