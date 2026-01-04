use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject};

use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::plex_rs::sync_playlist::MissingTrack;

#[derive(Debug, Clone, SimpleObject)]
pub struct SyncPlaylistToPlexResult {
    pub missing_tracks: Vec<MissingTrackInfo>,
    pub tracks_added: u32,
    pub tracks_removed: u32,
    pub tracks_skipped: u32,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct MissingTrackInfo {
    pub track_id: i64,
    pub file_path: String,
    pub title: String,
}

impl From<MissingTrack> for MissingTrackInfo {
    fn from(track: MissingTrack) -> Self {
        MissingTrackInfo {
            track_id: track.track_id,
            file_path: track.file_path,
            title: track.title,
        }
    }
}

#[derive(Default)]
pub struct PlexPlaylistMutation;

#[Object]
impl PlexPlaylistMutation {
    /// Sync a database playlist to Plex
    async fn sync_playlist_to_plex(
        &self,
        ctx: &Context<'_>,
        playlist_id: i64,
    ) -> GraphqlResult<SyncPlaylistToPlexResult> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let client = reqwest::Client::new();
        let result =
            crate::plex_rs::sync_playlist::sync_playlist_to_plex(db, &client, playlist_id).await?;

        Ok(SyncPlaylistToPlexResult {
            missing_tracks: result
                .missing_tracks
                .into_iter()
                .map(MissingTrackInfo::from)
                .collect(),
            tracks_added: result.tracks_added,
            tracks_removed: result.tracks_removed,
            tracks_skipped: result.tracks_skipped,
        })
    }
}
