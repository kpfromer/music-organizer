use ratatui::{
    prelude::*,
    widgets::{Block, Borders, ListItem, Paragraph},
};

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

/// Format duration in seconds to MM:SS format
pub fn format_duration(seconds: u32) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

/// Format speed in bytes per second
#[allow(dead_code)]
pub fn format_speed(bytes_per_sec: f64) -> String {
    format_size(bytes_per_sec as u64) + "/s"
}

/// Render an input field with focus indicator
pub fn render_input_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_focused: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(label)
        .border_style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let text = if value.is_empty() {
        " ".to_string()
    } else {
        value.to_string()
    };

    let paragraph = Paragraph::new(text).block(block).style(if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    });

    frame.render_widget(paragraph, area);
}

/// Render a result item
pub fn render_result_item(result: &song_rs::SongResult, is_selected: bool) -> ListItem<'_> {
    let mut lines = vec![];

    // Main line: filename
    let prefix = if is_selected { "> " } else { "  " };
    let style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    lines.push(Line::from(vec![
        Span::styled(prefix, style),
        Span::styled(result.filename.as_str().to_string(), style),
    ]));

    // Metadata line
    let mut metadata = vec![];

    metadata.push(format!("Size: {}", format_size(result.size)));

    if let Some(duration) = result.duration {
        metadata.push(format!("Duration: {}", format_duration(duration)));
    }

    if let Some(bitrate) = result.bitrate {
        metadata.push(format!("Bitrate: {} kbps", bitrate));
    }

    if !metadata.is_empty() {
        lines.push(Line::from(format!("     {}", metadata.join(" | "))));
    }

    let user_info = format!("     User: {}", result.username);
    lines.push(Line::from(user_info));

    ListItem::new(lines)
}
