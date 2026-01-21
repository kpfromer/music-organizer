use std::collections::HashMap;

use super::types::Candidate;
use crate::{database::Database, entities};
use color_eyre::eyre::{Context, OptionExt, Result};
use regex::Regex;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, EntityTrait};

/// Remove punctuation/special chars using regex, then collapse spaces
fn normalize(string: &str) -> Result<String> {
    let re_punct = Regex::new(r"[^\p{L}\p{N}\s]+").wrap_err("Failed to create regex")?;
    let re_space = Regex::new(r"\s+").wrap_err("Failed to create regex")?;
    let lower_trim = string.to_lowercase().trim().to_string();
    let no_punct = re_punct.replace_all(&lower_trim, "");
    let collapsed = re_space.replace_all(&no_punct, " ");
    Ok(collapsed.to_string())
}

fn compute_field_confidence(field1: &str, field2: &str) -> Result<f64> {
    let normalized1 = normalize(field1)?;
    let normalized2 = normalize(field2)?;
    let similarity = jaro_winkler::jaro_winkler(&normalized1, &normalized2);
    Ok(similarity)
}

fn similarity_score(c1: &Candidate, c2: &Candidate) -> Result<f64> {
    let title_sim = compute_field_confidence(&c1.title, &c2.title)?;
    let artist_sim = compute_field_confidence(&c1.artist, &c2.artist)?;
    let album_sim = compute_field_confidence(&c1.album, &c2.album)?;

    // Combine them into a single confidence score
    // You could adjust weights for each field if you like, e.g.:
    // const finalScore = 0.5 * titleSim + 0.3 * artistSim + 0.2 * albumSim;    const final_score = (title_sim + artist_sim + album_sim) / 3;
    let final_score = (title_sim + artist_sim + album_sim) / 3.0;
    Ok(final_score)
}

async fn get_local_tracks_and_candidates(db: &Database) -> Result<Vec<(i64, Candidate)>> {
    let all_local_tracks = entities::track::Entity::find()
        .find_also_related(entities::album::Entity)
        .all(&db.conn)
        .await?;

    let mut candidates = Vec::new();

    for (track, album) in all_local_tracks {
        log::debug!("Processing local track: {:?}", track);
        let album = album.ok_or_eyre("No album found for track")?;
        // TODO: fix this n+1 query
        let primary_artist = entities::album_artist::Entity::find()
            .filter(entities::album_artist::Column::AlbumId.eq(album.id))
            .filter(entities::album_artist::Column::IsPrimary.eq(1))
            .find_also_related(entities::artist::Entity)
            .one(&db.conn)
            .await?
            .ok_or_eyre("No primary artist found for album")?
            .1
            .ok_or_eyre("No artist found for primary artist link")?;

        let candidate = Candidate {
            title: track.title.clone(),
            album: album.title.clone(),
            artist: primary_artist.name,
            duration: track.duration.map(|d| d * 1000),
        };
        candidates.push((track.id, candidate));
    }

    Ok(candidates)
}

pub async fn filter_for_best_local_matches<'a>(
    db: &Database,
    spotify_tracks: &'a [entities::spotify_track::Model],
) -> Result<Vec<(&'a entities::spotify_track::Model, Vec<(i64, Candidate)>)>> {
    let all_local_track_candidates = get_local_tracks_and_candidates(db).await?;
    let local_track_id_to_local_track_candidate = all_local_track_candidates
        .iter()
        .map(|(id, candidate)| (*id, candidate))
        .collect::<HashMap<i64, &Candidate>>();

    let mut best_local_matches = Vec::new();

    for spotify_track in spotify_tracks {
        let spotify_track_candidate = Candidate {
            title: spotify_track.title.clone(),
            artist: spotify_track
                .artists
                .0
                .first()
                .ok_or_eyre("No artist found for spotify track")?
                .clone(),
            album: spotify_track.album.clone(),
            duration: spotify_track.duration,
        };

        log::debug!(
            "Computing similarity scores for spotify track: {:?}",
            spotify_track_candidate
        );
        let mut similarity_scores = all_local_track_candidates
            .iter()
            .map(|(id, candidate)| -> Result<(f64, i64)> {
                Ok((similarity_score(&spotify_track_candidate, candidate)?, *id))
            })
            .collect::<Result<Vec<_>>>()
            .wrap_err("Failed to compute similarity scores")?;
        similarity_scores.sort_by(|a, b| b.0.total_cmp(&a.0));
        let best_local_track_id_matches = similarity_scores
            .into_iter()
            .map(|(_score, id)| id)
            .take(10)
            .collect::<Vec<_>>();

        log::debug!(
            "Best local track id matches: {:?}",
            best_local_track_id_matches
        );
        let best_local_track_matches = best_local_track_id_matches
            .into_iter()
            .map(|id| -> Result<(i64, Candidate)> {
                Ok((
                    id,
                    local_track_id_to_local_track_candidate
                        .get(&id)
                        .map(|&c| c.clone())
                        .ok_or_eyre("No local track candidate found for id")?,
                ))
            })
            .collect::<Result<Vec<_>>>()
            .wrap_err("Failed to get best local track matches")?;

        best_local_matches.push((spotify_track, best_local_track_matches));
    }

    Ok(best_local_matches)
}
