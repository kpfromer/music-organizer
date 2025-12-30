use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create playlists table
        manager
            .create_table(
                Table::create()
                    .table(Playlist::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Playlist::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Playlist::Name).string().not_null())
                    .col(ColumnDef::new(Playlist::Description).string())
                    .col(ColumnDef::new(Playlist::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Playlist::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // Create playlist_tracks junction table
        manager
            .create_table(
                Table::create()
                    .table(PlaylistTrack::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlaylistTrack::PlaylistId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PlaylistTrack::TrackId).integer().not_null())
                    .col(
                        ColumnDef::new(PlaylistTrack::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PlaylistTrack::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(PlaylistTrack::PlaylistId)
                            .col(PlaylistTrack::TrackId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_tracks_playlist_id")
                            .from(PlaylistTrack::Table, PlaylistTrack::PlaylistId)
                            .to(Playlist::Table, Playlist::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_tracks_track_id")
                            .from(PlaylistTrack::Table, PlaylistTrack::TrackId)
                            .to(Track::Table, Track::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(PlaylistTrack::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Playlist::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Playlist {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum PlaylistTrack {
    Table,
    PlaylistId,
    TrackId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Track {
    Table,
    Id,
}
