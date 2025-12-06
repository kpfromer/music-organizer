use crate::soulseek_tui::widgets;
use crate::soulseek_tui::{app::App, widgets::format_size};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, Paragraph},
};

pub fn render(frame: &mut Frame, app: &App) {
    match app.mode {
        crate::soulseek_tui::app::AppMode::SearchForm => render_search_form(frame, app),
        crate::soulseek_tui::app::AppMode::Results => render_results(frame, app),
        crate::soulseek_tui::app::AppMode::Downloading => render_download_progress(frame, app),
        crate::soulseek_tui::app::AppMode::Error => render_error(frame, app),
    }
}

fn render_search_form(frame: &mut Frame, app: &App) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(0),    // Form fields
            Constraint::Length(3), // Status bar
            Constraint::Length(1), // Help text
        ])
        .split(area);

    // Header
    let title = Paragraph::new("SoulSeek Downloader").style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_widget(title, chunks[0]);

    // Form fields
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Artist
            Constraint::Length(3), // Album
            Constraint::Length(3), // Length
            Constraint::Min(0),    // Spacer
        ])
        .split(chunks[1]);

    widgets::render_input_field(
        frame,
        form_chunks[0],
        "Title",
        &app.form.title,
        matches!(
            app.form.focused_field,
            crate::soulseek_tui::app::FormField::Title
        ),
    );

    widgets::render_input_field(
        frame,
        form_chunks[1],
        "Artist",
        &app.form.artist,
        matches!(
            app.form.focused_field,
            crate::soulseek_tui::app::FormField::Artist
        ),
    );

    widgets::render_input_field(
        frame,
        form_chunks[2],
        "Album",
        &app.form.album,
        matches!(
            app.form.focused_field,
            crate::soulseek_tui::app::FormField::Album
        ),
    );

    widgets::render_input_field(
        frame,
        form_chunks[3],
        "Length (seconds)",
        &app.form.length,
        matches!(
            app.form.focused_field,
            crate::soulseek_tui::app::FormField::Length
        ),
    );

    // Status bar
    let status = app.status_message.as_deref().unwrap_or("Ready");
    let status_para =
        Paragraph::new(format!("Status: {}", status)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(status_para, chunks[2]);

    // Help text
    let help = Paragraph::new("[Enter: Search] [Tab: Next Field] [q: Quit]")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);
}

fn render_results(frame: &mut Frame, app: &App) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Results list
            Constraint::Length(3), // Help text
            Constraint::Length(3), // Output path
        ])
        .split(area);

    // Header
    let header_text = format!("Search Results ({} found)", app.results.len());
    let header = Block::default()
        .borders(Borders::ALL)
        .title(header_text)
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(header, chunks[0]);

    // Results list
    if !app.results.is_empty() {
        let items: Vec<_> = app
            .results
            .iter()
            .enumerate()
            .skip(app.results_scroll)
            .take(chunks[1].height as usize - 2)
            .map(|(idx, result)| widgets::render_result_item(result, idx == app.selected_result))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default());
        frame.render_widget(list, chunks[1]);
    } else {
        let empty = Paragraph::new("No results found")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, chunks[1]);
    }

    // Help text
    let help = Paragraph::new("[Enter: Download] [Esc: Back] [↑↓: Navigate] [q: Quit]")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::bordered().title("Help"));
    frame.render_widget(help, chunks[2]);

    // Output path
    let output_text = format!("Output: {}", app.output_directory.display());
    let output = Paragraph::new(output_text).block(Block::bordered().title("Output"));

    frame.render_widget(output, chunks[3]);
}

fn render_download_progress(frame: &mut Frame, app: &App) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(5), // Progress bar
            Constraint::Min(0),    // Spacer
            Constraint::Length(3), // Help text
        ])
        .split(area);

    // Header
    let header = Block::default()
        .borders(Borders::ALL)
        .title("Downloading...")
        .title_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(header, chunks[0]);

    // Progress
    if let Some(ref progress) = app.download_progress {
        let label = Span::styled(
            format!(
                "{}/{}",
                format_size(progress.bytes_downloaded),
                format_size(progress.total_bytes),
            ),
            Style::new().italic().bold().fg(Color::Green),
        );

        let progress_gauge = Gauge::default()
            .label(label)
            .block(Block::bordered().title("Progress"))
            .ratio(progress.bytes_downloaded as f64 / progress.total_bytes as f64);
        frame.render_widget(progress_gauge, chunks[1]);
    }

    // Help text
    let help = Paragraph::new("[Esc: Cancel]").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);
}

fn render_error(frame: &mut Frame, app: &App) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Error message
            Constraint::Length(1), // Help text
        ])
        .split(area);

    // Header
    let header = Block::default()
        .borders(Borders::ALL)
        .title("Error")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);

    // Error message
    let error_text = app.error_message.as_deref().unwrap_or("Unknown error");
    let error_para = Paragraph::new(error_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Red))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(error_para, chunks[1]);

    // Help text
    let help = Paragraph::new("[Enter/Esc: Dismiss]").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);
}
