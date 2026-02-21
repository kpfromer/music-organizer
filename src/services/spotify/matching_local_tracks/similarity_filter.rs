use super::matcher::{MatchResult, Track, find_matches};
use crate::{database::Database, entities};
use color_eyre::eyre::{OptionExt, Result};
use rayon::prelude::*;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, EntityTrait};

pub fn spotify_track_to_track(spotify_track: &entities::spotify_track::Model) -> Result<Track> {
    Ok(Track {
        title: spotify_track.title.clone(),
        primary_artist: spotify_track
            .artists
            .0
            .first()
            .ok_or_eyre("No artist found for spotify track")?
            .clone(),
        secondary_artists: spotify_track.artists.0[1..].to_vec(),
        album: spotify_track.album.clone(),
        duration_ms: spotify_track
            .duration
            .ok_or_eyre("No duration found for spotify track")? as u32,
    })
}

pub async fn db_track_to_track(db: &Database, track: &entities::track::Model) -> Result<Track> {
    let album = entities::album::Entity::find_by_id(track.album_id)
        .one(&db.conn)
        .await?
        .ok_or_eyre("No album found for track")?;
    let primary_artist = entities::album_artist::Entity::find()
        .filter(entities::album_artist::Column::AlbumId.eq(track.album_id))
        .filter(entities::album_artist::Column::IsPrimary.eq(1))
        .find_also_related(entities::artist::Entity)
        .one(&db.conn)
        .await?
        .ok_or_eyre("No primary artist found for album")?
        .1
        .ok_or_eyre("No artist found for primary artist link")?
        .name;
    let secondary_artists = entities::album_artist::Entity::find()
        .filter(entities::album_artist::Column::AlbumId.eq(track.album_id))
        .filter(entities::album_artist::Column::IsPrimary.eq(0))
        .find_also_related(entities::artist::Entity)
        .all(&db.conn)
        .await?
        .into_iter()
        .filter_map(|(_, artist)| artist.map(|a| a.name))
        .collect::<Vec<_>>();

    Ok(Track {
        title: track.title.clone(),
        primary_artist,
        secondary_artists,
        album: album.title.clone(),
        duration_ms: track.duration.ok_or_eyre("No duration found for track")? as u32 * 1000,
    })
}

pub async fn match_spotify_track_to_local_track<'a>(
    db: &Database,
    spotify_tracks: &'a [entities::spotify_track::Model],
    local_tracks: &[entities::track::Model],
) -> Result<
    Vec<(
        &'a entities::spotify_track::Model,
        Vec<(entities::track::Model, MatchResult)>,
    )>,
> {
    let spotify_tracks_tracks = spotify_tracks
        .iter()
        .map(spotify_track_to_track)
        .collect::<Result<Vec<_>>>()?;
    let local_tracks_tracks = {
        let mut local_tracks_tracks = Vec::new();
        for local_track in local_tracks.iter() {
            local_tracks_tracks.push(db_track_to_track(db, local_track).await?);
        }
        local_tracks_tracks
    };

    let match_results = spotify_tracks_tracks
        .into_iter()
        .zip(spotify_tracks.iter())
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|(spotify_track_track, spotify_track)| {
            (
                spotify_track,
                find_matches(&spotify_track_track, &local_tracks_tracks, 0.5),
            )
        })
        .collect::<Vec<_>>();

    let match_results = match_results
        .into_iter()
        .map(
            |(spotify_track, matches)| -> Result<(
                &entities::spotify_track::Model,
                Vec<(entities::track::Model, MatchResult)>,
            )> {
                Ok((
                    spotify_track,
                    matches
                        .into_iter()
                        .map(|(index, _, match_result)| -> Result<(entities::track::Model, MatchResult)> {
                            Ok((
                                local_tracks
                                    .get(index)
                                    .ok_or_eyre("No local track found for index")?
                                    .clone(),
                                match_result,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                ))
            },
        )
        .collect::<Result<Vec<_>>>()?;

    Ok(match_results)
}
