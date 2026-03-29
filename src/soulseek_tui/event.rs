use color_eyre::eyre::Context;
/// This file is based on https://github.com/ratatui/templates/blob/main/event-driven/template/src/event.rs
use ratatui::crossterm::event::{self, Event as CrosstermEvent};
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::Duration,
};
use tracing;

use std::sync::Arc;

use song_rs::{Client as SongDownloader, SongQuery, SongResult};
use tokio::sync::Mutex;

const TIMEOUT: Duration = Duration::from_millis(250);

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
    /// Background events.
    ///
    /// These events are emitted by the background thread.
    Background(BackgroundEvent),
}

/// Application events.
///
/// You can extend this enum with your own custom events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Start a soulseek search.
    StartSearch,
    /// Start a download.
    StartDownload,
}

/// Background events.
///
/// These events are emitted by the background thread.
#[derive(Clone, Debug)]
pub enum BackgroundEvent {
    /// Search event.
    SearchEvent(SearchEvent),
    /// Download event.
    DownloadEvent(DownloadEvent),
}

#[derive(Clone, Debug)]
pub enum SearchEvent {
    /// Search started.
    Started,
    /// Search completed.
    Completed(Vec<SongResult>),
    /// Search failed.
    Failed(String),
}

/// Download events.
///
/// These events are emitted by the download thread.
#[derive(Clone, Debug)]
pub enum DownloadEvent {
    /// Download started.
    Started,
    /// Download progress.
    Progress {
        filename: String,
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    /// Download completed.
    Completed,
    /// Download failed.
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct SearchRequest {
    /// Query to search for.
    pub query: SongQuery,
}

#[derive(Clone, Debug)]
pub struct RequestDownload {
    /// Result to download.
    pub result: SongResult,
    /// Download path.
    pub download_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum BackgroundRequest {
    /// Search for a track.
    Search(SearchRequest),
    /// Download a file.
    Download(RequestDownload),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Background sender channel.
    background_sender: mpsc::Sender<BackgroundRequest>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new(song_downloader: Arc<Mutex<SongDownloader>>) -> Self {
        let (sender, receiver) = mpsc::channel();

        let cross_term_actor = CrosstermEventThread::new(sender.clone());
        thread::spawn(|| cross_term_actor.run());

        let (background_sender, background_receiver) = mpsc::channel();
        let download_actor =
            BackgroundThread::new(background_receiver, sender.clone(), song_downloader);
        thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(download_actor.run()) {
                tracing::error!("Background thread error: {}", e);
            }
        });

        Self {
            sender,
            receiver,
            background_sender,
        }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub fn next(&self) -> color_eyre::Result<Event> {
        self.receiver.recv().context("failed to receive event")
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }

    /// Queue a download request to be sent to the download thread.
    pub fn send_background_request(&mut self, request: BackgroundRequest) {
        tracing::debug!("Sending background request: {:?}", request);
        let _ = self.background_sender.send(request);
    }
}

/// A thread that handles reading crossterm events
struct CrosstermEventThread {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
}

impl CrosstermEventThread {
    /// Constructs a new instance of [`CrosstermEventThread`].
    fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    fn run(self) -> color_eyre::Result<()> {
        loop {
            if event::poll(TIMEOUT).context("failed to poll for crossterm events")? {
                let event = event::read().context("failed to read crossterm event")?;
                self.send(Event::Crossterm(event));
            }
        }
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}

struct BackgroundThread {
    /// Background request receiver channel.
    background_request_receiver: mpsc::Receiver<BackgroundRequest>,
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Song downloader client.
    song_downloader: Arc<Mutex<SongDownloader>>,
}

impl BackgroundThread {
    /// Constructs a new instance of [`BackgroundThread`].
    fn new(
        background_request_receiver: mpsc::Receiver<BackgroundRequest>,
        sender: mpsc::Sender<Event>,
        song_downloader: Arc<Mutex<SongDownloader>>,
    ) -> Self {
        Self {
            background_request_receiver,
            sender,
            song_downloader,
        }
    }

    /// Runs the background thread.
    async fn run(mut self) -> color_eyre::Result<()> {
        loop {
            match self.background_request_receiver.recv() {
                Ok(BackgroundRequest::Search(_request)) => self.handle_search(_request).await,
                Ok(BackgroundRequest::Download(_request)) => self.handle_download(_request).await,
                Err(_) => {
                    tracing::debug!(
                        "Background request receiver disconnected, shutting down background thread"
                    );
                    return Ok(());
                }
            }
        }
    }

    async fn handle_search(&mut self, request: SearchRequest) {
        let _ = self
            .sender
            .send(Event::Background(BackgroundEvent::SearchEvent(
                SearchEvent::Started,
            )));

        let results = {
            let guard = self.song_downloader.lock().await;
            guard
                .search(
                    &request.query,
                    Duration::from_secs(120),
                    &song_rs::WantedFileTypes::all(),
                )
                .await
        };

        match results {
            Ok(results) => {
                let _ = self
                    .sender
                    .send(Event::Background(BackgroundEvent::SearchEvent(
                        SearchEvent::Completed(results),
                    )));
            }
            Err(e) => {
                let _ = self
                    .sender
                    .send(Event::Background(BackgroundEvent::SearchEvent(
                        SearchEvent::Failed(e.to_string()),
                    )));
            }
        }
    }

    async fn download_file(
        &mut self,
        result: &SongResult,
        download_folder: &Path,
    ) -> color_eyre::Result<()> {
        let download_dir = download_folder
            .as_os_str()
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Download path is not valid UTF-8"))?
            .to_string();

        let (_download, mut receiver) = {
            let guard = self.song_downloader.lock().await;
            guard
                .download(result, &download_dir, Some(Duration::from_secs(30)))
                .await
        }?;

        let filename_str = result.filename.filename().to_string();

        while let Some(status) = receiver.recv().await {
            match status {
                song_rs::DownloadStatus::Queued => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Started,
                        )))?;
                }
                song_rs::DownloadStatus::InProgress {
                    bytes_downloaded,
                    total_bytes,
                    ..
                } => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Progress {
                                filename: filename_str.clone(),
                                bytes_downloaded,
                                total_bytes,
                            },
                        )))?;
                }
                song_rs::DownloadStatus::Completed => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Completed,
                        )))?;
                    break;
                }
                song_rs::DownloadStatus::Failed => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Failed("Download failed".to_string()),
                        )))?;
                    break;
                }
                song_rs::DownloadStatus::TimedOut => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Failed("Download timed out".to_string()),
                        )))?;
                    break;
                }
                song_rs::DownloadStatus::Cancelled => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Failed("Download was cancelled".to_string()),
                        )))?;
                    break;
                }
            }
        }
        tracing::debug!("Download file thread finished");
        Ok(())
    }

    async fn handle_download(&mut self, request: RequestDownload) {
        match self
            .download_file(&request.result, &request.download_path)
            .await
        {
            Ok(_) => {
                let _ = self
                    .sender
                    .send(Event::Background(BackgroundEvent::DownloadEvent(
                        DownloadEvent::Completed,
                    )));
            }
            Err(e) => {
                let _ = self
                    .sender
                    .send(Event::Background(BackgroundEvent::DownloadEvent(
                        DownloadEvent::Failed(e.to_string()),
                    )));
            }
        }
    }
}
