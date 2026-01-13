use super::types::Candidate;
use crate::entities;
use color_eyre::eyre::{Context, Result};
use ollama_native::Ollama;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

pub async fn pick_best_local_match(
    spotify_track: &entities::spotify_track::Model,
    local_track_candidates: Vec<(i64, Candidate)>,
) -> Result<Option<i64>> {
    #[derive(Serialize)]
    struct LocalTrackSelectionRequest {
        id: i64,
        title: String,
        artist: String,
        album: String,
        duration_ms: String,
    }

    #[derive(JsonSchema, Deserialize)]
    struct LocalTrackSelectionResponse {
        best_local_track_id: Option<i64>,
        reason: String,
    }

    // TODO: use ollama url argument
    log::debug!("Using ollama at http://10.4.0.20:7869 to pick best local track");
    let ollama = Ollama::new("http://10.4.0.20:7869");
    let json_schema = schema_for!(LocalTrackSelectionResponse);
    let json_schema_str = serde_json::to_string_pretty(&json_schema)
        .wrap_err("Failed to convert JSON schema to string")?;

    let request_json = serde_json::to_string_pretty(
        &local_track_candidates
            .into_iter()
            .map(|(track_id, track)| LocalTrackSelectionRequest {
                id: track_id,
                title: track.title,
                artist: track.artist,
                album: track.album,
                duration_ms: track
                    .duration
                    .map(|d| format!("{d} milliseconds"))
                    .unwrap_or("not specified".to_string()),
            })
            .collect::<Vec<_>>(),
    )
    .wrap_err("Failed to convert local track selection request to string")?;

    let prompt = format!(
        r#"
You are a music track re-ranker. Your task is to select the single best local track from a pre-filtered list of candidate tracks that have already been identified as potential matches for a Spotify track.

The candidates have been pre-filtered using similarity scoring, but you should use your understanding of music metadata to make the final selection. Consider:

1. Title matching (most important)
   - Exact matches are ideal
   - Handle variations: "Don't Stop" vs "Dont Stop", "The Beatles" vs "Beatles, The"
   - Be aware of remixes, live versions, and alternate takes (these are different tracks)

2. Artist matching (very important)
   - Primary artist should match
   - Handle variations: "Artist A feat. Artist B" vs "Artist A", "Artist A & Artist B" vs "Artist A"
   - Consider featured artists, but primary artist match is most important

3. Album matching (important)
   - Helps confirm it's the same recording
   - Different albums may indicate different versions (single vs album version, remix album, etc.)
   - Missing album info is acceptable if title and artist match well

4. Duration (tiebreaker only)
   - Small differences (< 5 seconds) are normal due to encoding/formatting
   - Large differences may indicate a different version (radio edit, extended mix, etc.)
   - Don't reject good matches solely due to duration differences

5. Edge cases to consider
   - Remixes: "Song (Remix)" vs "Song" - these are different tracks
   - Live versions: "Song (Live)" vs "Song" - these are different tracks
   - Instrumental/Karaoke versions - these are different tracks
   - Different releases: single version vs album version (usually same track, different metadata)

Return the best_local_track_id if you find a good match. Return null only if:
- None of the candidates appear to be the same track (all are clearly different songs)
- The matches are clearly different recordings (e.g., remix vs original, live vs studio)

Spotify track to match:
title: {title}
artist: {artist}
album: {album}
duration: {duration}

Pre-filtered candidate local tracks (already identified as potential matches):
{local_tracks}
    "#,
        title = spotify_track.title,
        artist = spotify_track.artists.0.first().unwrap().clone(),
        album = spotify_track.album,
        duration = spotify_track
            .duration
            .map(|d| format!("{d} milliseconds"))
            .unwrap_or("not specified".to_string()),
        local_tracks = request_json,
    );
    let response = ollama
        .generate("nemotron-3-nano:30b")
        .prompt(&prompt)
        .format(&json_schema_str)
        .await?;
    let response_json = serde_json::from_str::<LocalTrackSelectionResponse>(&response.response)?;

    if response_json.best_local_track_id.is_none() {
        log::error!(
            "No best local track found for spotify track: {:?}. Reason: {}",
            spotify_track,
            response_json.reason
        );
    }

    Ok(response_json.best_local_track_id)
}
