use anyhow::Context;
/// This file is based on https://github.com/ratatui/templates/blob/main/event-driven/template/src/event.rs
use ratatui::crossterm::event::{self, Event as CrosstermEvent};
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

use crate::soulseek::{SingleFileResult, SoulSeekClientContext, Track};

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
    Completed(Vec<SingleFileResult>),
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
    /// Track to search for.
    pub track: Track,
}

#[derive(Clone, Debug)]
pub struct RequestDownload {
    /// Result to download.
    pub result: SingleFileResult,
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
    pub fn new(soulseek_context: SoulSeekClientContext) -> Self {
        let (sender, receiver) = mpsc::channel();

        let cross_term_actor = CrosstermEventThread::new(sender.clone());
        thread::spawn(|| cross_term_actor.run());

        let (background_sender, background_receiver) = mpsc::channel();
        let download_actor =
            BackgroundThread::new(background_receiver, sender.clone(), soulseek_context);
        thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(download_actor.run()).unwrap();
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
    pub fn next(&self) -> anyhow::Result<Event> {
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
    fn run(self) -> anyhow::Result<()> {
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
    /// Soulseek context.
    soulseek_context: SoulSeekClientContext,
}

impl BackgroundThread {
    /// Constructs a new instance of [`BackgroundThread`].
    fn new(
        background_request_receiver: mpsc::Receiver<BackgroundRequest>,
        sender: mpsc::Sender<Event>,
        soulseek_context: SoulSeekClientContext,
    ) -> Self {
        Self {
            background_request_receiver,
            sender,
            soulseek_context,
        }
    }

    /// Runs the background thread.
    async fn run(mut self) -> anyhow::Result<()> {
        loop {
            match self.background_request_receiver.recv() {
                Ok(BackgroundRequest::Search(_request)) => self.handle_search(_request).await,
                Ok(BackgroundRequest::Download(_request)) => self.handle_download(_request).await,
                Err(_) => {}
            }
        }
    }

    async fn handle_search(&mut self, request: SearchRequest) {
        let _ = self
            .sender
            .send(Event::Background(BackgroundEvent::SearchEvent(
                SearchEvent::Started,
            )));

        match crate::soulseek::search_for_track(&request.track, &mut self.soulseek_context).await {
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
        result: &SingleFileResult,
        download_folder: &PathBuf,
    ) -> anyhow::Result<()> {
        let receiver =
            crate::soulseek::download_file(&result, download_folder, &mut self.soulseek_context)
                .await?;

        for status in receiver {
            match status {
                soulseek_rs::DownloadStatus::Queued => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Started,
                        )))?;
                }
                soulseek_rs::DownloadStatus::InProgress {
                    bytes_downloaded,
                    total_bytes,
                    speed_bytes_per_sec: _,
                } => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Progress {
                                filename: result.filename.clone(),
                                bytes_downloaded,
                                total_bytes,
                            },
                        )))?;
                }
                soulseek_rs::DownloadStatus::Completed => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Completed,
                        )))?;
                }
                soulseek_rs::DownloadStatus::Failed => {
                    self.sender
                        .send(Event::Background(BackgroundEvent::DownloadEvent(
                            DownloadEvent::Failed("Download failed".to_string()),
                        )))?;
                }
                soulseek_rs::DownloadStatus::TimedOut => {}
            }
        }
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
