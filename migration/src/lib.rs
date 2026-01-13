pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_tables;
mod m20251216_050017_unimportable_file;
mod m20251230_021238_create_playlist_table;
mod m20251230_165711_create_plex_servers_table;
mod m20260107_000001_add_track_identifiers;
mod m20260107_000002_create_spotify_tables;
mod m20260109_194726_add_irscs_to_tracks;
mod m20260113_012025_create_spotify_track;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tables::Migration),
            Box::new(m20251216_050017_unimportable_file::Migration),
            Box::new(m20251230_021238_create_playlist_table::Migration),
            Box::new(m20251230_165711_create_plex_servers_table::Migration),
            Box::new(m20260107_000001_add_track_identifiers::Migration),
            Box::new(m20260107_000002_create_spotify_tables::Migration),
            Box::new(m20260109_194726_add_irscs_to_tracks::Migration),
            Box::new(m20260113_012025_create_spotify_track::Migration),
        ]
    }
}
