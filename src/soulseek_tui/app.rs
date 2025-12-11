use crate::config::Config;
use crate::soulseek::{SearchConfig, SingleFileResult, SoulSeekClientContext, Track};
use crate::soulseek_tui::event::{
    AppEvent, BackgroundEvent, BackgroundRequest, DownloadEvent, Event, EventHandler,
    RequestDownload, SearchEvent, SearchRequest,
};
use crate::soulseek_tui::input::handle_key_event;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    SearchForm,
    Results,
    Downloading,
    Error,
}

#[derive(Debug, Clone)]
pub enum FormField {
    Title,
    Artist,
    Album,
    Length,
}

#[derive(Debug, Clone)]
pub struct SearchForm {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub length: String, // String for input, parse to u32
    pub focused_field: FormField,
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub filename: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Starting,
    InProgress,
    Complete,
    Failed(String),
}

pub struct App {
    pub mode: AppMode,
    pub form: SearchForm,
    pub results: Vec<SingleFileResult>,
    pub selected_result: usize,
    pub results_scroll: usize,
    pub download_progress: Option<DownloadProgress>,
    pub soulseek_context: Option<SoulSeekClientContext>,
    pub search_config: SearchConfig,
    pub output_directory: PathBuf,
    pub error_message: Option<String>,
    pub status_message: Option<String>,
    pub pending_search: bool,
    pub pending_download: bool,

    /// Event handler.
    pub running: bool,
    pub events: EventHandler,
}

impl App {
    pub fn new(
        config: &Config,
        soulseek_context: SoulSeekClientContext,
        download_output_directory: PathBuf,
    ) -> Self {
        // TODO: take in from config
        let search_config = SearchConfig {
            // TODO: REMOVE
            username: "".to_string(),
            password: "".to_string(),
            // username: soulseek_config.username,
            // password: soulseek_config.password,
            concurrency: Some(2),
            searches_per_time: Some(34),
            renew_time_secs: Some(220),
            max_search_time_ms: Some(8000),
            remove_special_chars: Some(true),
        };

        Self {
            mode: AppMode::SearchForm,
            form: SearchForm {
                title: String::new(),
                artist: String::new(),
                album: String::new(),
                length: String::new(),
                focused_field: FormField::Title,
            },
            results: Vec::new(),
            selected_result: 0,
            results_scroll: 0,
            download_progress: None,
            soulseek_context: None,
            search_config,
            output_directory: download_output_directory,
            error_message: None,
            status_message: Some("Ready".to_string()),
            pending_search: false,
            pending_download: false,
            running: true,
            events: EventHandler::new(soulseek_context),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut ratatui::prelude::Terminal<
            ratatui::prelude::CrosstermBackend<std::io::Stdout>,
        >,
    ) -> Result<()> {
        while self.running {
            // Render
            terminal.draw(|f| crate::soulseek_tui::ui::render(f, self))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match self.events.next()? {
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    handle_key_event(self, key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::StartSearch => {
                    self.events
                        .send_background_request(BackgroundRequest::Search(SearchRequest {
                            track: Track {
                                title: self.form.title.clone(),
                                artists: vec![self.form.artist.clone()],
                                album: self.form.album.clone(),
                                length: self.form.length.parse::<u32>().ok(),
                            },
                        }));
                }
                AppEvent::StartDownload => {
                    let result = self.results[self.selected_result].clone();

                    self.events
                        .send_background_request(BackgroundRequest::Download(RequestDownload {
                            result,
                            download_path: self.output_directory.clone(),
                        }));
                    self.mode = AppMode::Downloading;
                }
            },
            Event::Background(background_event) => match background_event {
                BackgroundEvent::SearchEvent(search_event) => match search_event {
                    SearchEvent::Started => {
                        self.status_message = Some("Searching...".to_string());
                    }
                    SearchEvent::Completed(results) => {
                        self.results = results;
                        self.selected_result = 0;
                        self.results_scroll = 0;
                        self.mode = AppMode::Results;
                        self.status_message = Some(format!("Found {} results", self.results.len()));
                    }
                    SearchEvent::Failed(error) => {
                        self.error_message = Some(error);
                        self.mode = AppMode::Error;
                    }
                },
                BackgroundEvent::DownloadEvent(download_event) => match download_event {
                    DownloadEvent::Started => {
                        self.status_message = Some("Download started".to_string())
                    }
                    DownloadEvent::Progress {
                        filename,
                        bytes_downloaded,
                        total_bytes,
                    } => {
                        self.download_progress = Some(DownloadProgress {
                            filename,
                            bytes_downloaded,
                            total_bytes,
                            status: DownloadStatus::InProgress,
                        })
                    }
                    DownloadEvent::Completed => {
                        self.status_message = Some("Download completed".to_string())
                    }
                    DownloadEvent::Failed(error) => self.error_message = Some(error),
                },
            },
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
