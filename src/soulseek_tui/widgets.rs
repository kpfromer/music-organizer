use crate::soulseek::{FileAttribute, SingleFileResult};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, ListItem, Paragraph},
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
pub fn render_result_item(result: &SingleFileResult, is_selected: bool) -> ListItem<'_> {
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
        Span::styled(result.filename.clone(), style),
    ]));

    // Metadata line
    let mut metadata = vec![];

    // Size
    metadata.push(format!("Size: {}", format_size(result.size)));

    // Duration if available
    if let Some(duration) = result.attrs.get(&FileAttribute::Duration) {
        metadata.push(format!("Duration: {}", format_duration(*duration)));
    }

    // Bitrate if available
    if let Some(bitrate) = result.attrs.get(&FileAttribute::Bitrate) {
        metadata.push(format!("Bitrate: {} kbps", bitrate));
    }

    if !metadata.is_empty() {
        lines.push(Line::from(format!("     {}", metadata.join(" | "))));
    }

    // User info line
    let user_info = format!(
        "     User: {} | Speed: {} | Free: {} | Queue: {}",
        result.username,
        format_speed(result.avg_speed),
        if result.slots_free { "✓" } else { "✗" },
        result.queue_length
    );
    lines.push(Line::from(user_info));

    ListItem::new(lines)
}

/// Render a progress bar for downloads
pub fn render_progress_bar(
    frame: &mut Frame,
    area: Rect,
    progress: &crate::soulseek_tui::app::DownloadProgress,
) {
    let percent = if progress.total_bytes > 0 {
        (progress.bytes_downloaded as f64 / progress.total_bytes as f64) * 100.0
    } else {
        0.0
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Download Progress");

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(Style::default().fg(Color::Green))
        .percent(percent as u16)
        .label(format!(
            "{} / {} ({:.1}%)",
            format_size(progress.bytes_downloaded),
            format_size(progress.total_bytes),
            percent
        ));

    frame.render_widget(gauge, area);
}
