use crate::soulseek_tui::{
    app::{App, AppMode, FormField},
    event::AppEvent,
};
use color_eyre::Result;
use crossterm::event::KeyEvent;

pub fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            Ok(())
        }
        _ => match app.mode {
            AppMode::SearchForm => handle_search_form_input(app, key),
            AppMode::Results => handle_results_input(app, key),
            AppMode::Downloading => handle_downloading_input(app, key),
            AppMode::Error => handle_error_input(app, key),
        },
    }
}

fn handle_search_form_input(app: &mut App, key: KeyEvent) -> Result<()> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.quit();
        }
        KeyCode::Tab | KeyCode::Down => {
            app.form.focused_field = match app.form.focused_field {
                FormField::Title => FormField::Artist,
                FormField::Artist => FormField::Album,
                FormField::Album => FormField::Length,
                FormField::Length => FormField::Title,
            };
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.form.focused_field = match app.form.focused_field {
                FormField::Title => FormField::Length,
                FormField::Artist => FormField::Title,
                FormField::Album => FormField::Artist,
                FormField::Length => FormField::Album,
            };
        }
        KeyCode::Enter => {
            app.events.send(AppEvent::StartSearch);
        }
        KeyCode::Char(c) => {
            let field = match app.form.focused_field {
                FormField::Title => &mut app.form.title,
                FormField::Artist => &mut app.form.artist,
                FormField::Album => &mut app.form.album,
                FormField::Length => &mut app.form.length,
            };
            field.push(c);
        }
        KeyCode::Backspace => {
            let field = match app.form.focused_field {
                FormField::Title => &mut app.form.title,
                FormField::Artist => &mut app.form.artist,
                FormField::Album => &mut app.form.album,
                FormField::Length => &mut app.form.length,
            };
            field.pop();
        }
        _ => {}
    }
    Ok(())
}

fn handle_results_input(app: &mut App, key: KeyEvent) -> Result<()> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::SearchForm;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_result > 0 {
                app.selected_result -= 1;
                // Adjust scroll if needed
                if app.selected_result < app.results_scroll {
                    app.results_scroll = app.selected_result;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_result < app.results.len().saturating_sub(1) {
                app.selected_result += 1;
                // Adjust scroll if needed
                let visible_height = 10; // Approximate visible items
                if app.selected_result >= app.results_scroll + visible_height {
                    app.results_scroll = app.selected_result - visible_height + 1;
                }
            }
        }
        KeyCode::PageUp => {
            if app.selected_result > 0 {
                app.selected_result = app.selected_result.saturating_sub(10);
                if app.selected_result < app.results_scroll {
                    app.results_scroll = app.selected_result;
                }
            }
        }
        KeyCode::PageDown => {
            let max_idx = app.results.len().saturating_sub(1);
            app.selected_result = (app.selected_result + 10).min(max_idx);
            let visible_height = 10;
            if app.selected_result >= app.results_scroll + visible_height {
                app.results_scroll = app.selected_result - visible_height + 1;
            }
        }
        KeyCode::Enter => {
            app.events.send(AppEvent::StartDownload);
        }
        _ => {}
    }
    Ok(())
}

fn handle_downloading_input(app: &mut App, key: KeyEvent) -> Result<()> {
    use crossterm::event::KeyCode;

    if key.code == KeyCode::Esc {
        app.mode = AppMode::Results;
    }
    Ok(())
}

fn handle_error_input(app: &mut App, key: KeyEvent) -> Result<()> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Enter | KeyCode::Esc => {
            app.mode = AppMode::SearchForm;
        }
        _ => {}
    }
    Ok(())
}
