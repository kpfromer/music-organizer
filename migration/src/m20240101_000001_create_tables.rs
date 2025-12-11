use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create artists table
        manager
            .create_table(
                Table::create()
                    .table(Artist::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Artist::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Artist::Name).string().not_null())
                    .col(ColumnDef::new(Artist::MusicbrainzId).string().unique_key())
                    .col(
                        ColumnDef::new(Artist::CreatedAt)
                            .integer()
                            .not_null()
                            .default(Expr::cust("(strftime('%s', 'now'))")),
                    )
                    .col(
                        ColumnDef::new(Artist::UpdatedAt)
                            .integer()
                            .not_null()
                            .default(Expr::cust("(strftime('%s', 'now'))")),
                    )
                    .to_owned(),
            )
            .await?;

        // Create albums table
        manager
            .create_table(
                Table::create()
                    .table(Album::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Album::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Album::Title).string().not_null())
                    .col(ColumnDef::new(Album::MusicbrainzId).string().unique_key())
                    .col(ColumnDef::new(Album::Year).integer())
                    .col(
                        ColumnDef::new(Album::CreatedAt)
                            .integer()
                            .not_null()
                            .default(Expr::cust("(strftime('%s', 'now'))")),
                    )
                    .col(
                        ColumnDef::new(Album::UpdatedAt)
                            .integer()
                            .not_null()
                            .default(Expr::cust("(strftime('%s', 'now'))")),
                    )
                    .to_owned(),
            )
            .await?;

        // Create tracks table
        manager
            .create_table(
                Table::create()
                    .table(Track::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Track::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Track::AlbumId).integer().not_null())
                    .col(ColumnDef::new(Track::Title).string().not_null())
                    .col(ColumnDef::new(Track::TrackNumber).integer())
                    .col(ColumnDef::new(Track::Duration).integer())
                    .col(ColumnDef::new(Track::MusicbrainzId).string().unique_key())
                    .col(
                        ColumnDef::new(Track::FilePath)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Track::Sha256)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Track::CreatedAt)
                            .integer()
                            .not_null()
                            .default(Expr::cust("(strftime('%s', 'now'))")),
                    )
                    .col(
                        ColumnDef::new(Track::UpdatedAt)
                            .integer()
                            .not_null()
                            .default(Expr::cust("(strftime('%s', 'now'))")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tracks_album_id")
                            .from(Track::Table, Track::AlbumId)
                            .to(Album::Table, Album::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Create album_artists junction table
        manager
            .create_table(
                Table::create()
                    .table(AlbumArtist::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AlbumArtist::AlbumId).integer().not_null())
                    .col(ColumnDef::new(AlbumArtist::ArtistId).integer().not_null())
                    .col(
                        ColumnDef::new(AlbumArtist::IsPrimary)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .primary_key(
                        Index::create()
                            .col(AlbumArtist::AlbumId)
                            .col(AlbumArtist::ArtistId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_album_artists_album_id")
                            .from(AlbumArtist::Table, AlbumArtist::AlbumId)
                            .to(Album::Table, Album::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_album_artists_artist_id")
                            .from(AlbumArtist::Table, AlbumArtist::ArtistId)
                            .to(Artist::Table, Artist::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create track_artists junction table
        manager
            .create_table(
                Table::create()
                    .table(TrackArtist::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TrackArtist::TrackId).integer().not_null())
                    .col(ColumnDef::new(TrackArtist::ArtistId).integer().not_null())
                    .col(
                        ColumnDef::new(TrackArtist::IsPrimary)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .primary_key(
                        Index::create()
                            .col(TrackArtist::TrackId)
                            .col(TrackArtist::ArtistId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_track_artists_track_id")
                            .from(TrackArtist::Table, TrackArtist::TrackId)
                            .to(Track::Table, Track::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_track_artists_artist_id")
                            .from(TrackArtist::Table, TrackArtist::ArtistId)
                            .to(Artist::Table, Artist::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_artists_musicbrainz_id")
                    .table(Artist::Table)
                    .col(Artist::MusicbrainzId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_albums_musicbrainz_id")
                    .table(Album::Table)
                    .col(Album::MusicbrainzId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tracks_musicbrainz_id")
                    .table(Track::Table)
                    .col(Track::MusicbrainzId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tracks_sha256")
                    .table(Track::Table)
                    .col(Track::Sha256)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tracks_file_path")
                    .table(Track::Table)
                    .col(Track::FilePath)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tracks_album_id")
                    .table(Track::Table)
                    .col(Track::AlbumId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_album_artists_album_id")
                    .table(AlbumArtist::Table)
                    .col(AlbumArtist::AlbumId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_album_artists_artist_id")
                    .table(AlbumArtist::Table)
                    .col(AlbumArtist::ArtistId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_track_artists_track_id")
                    .table(TrackArtist::Table)
                    .col(TrackArtist::TrackId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_track_artists_artist_id")
                    .table(TrackArtist::Table)
                    .col(TrackArtist::ArtistId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(TrackArtist::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AlbumArtist::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Track::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Album::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Artist::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Artist {
    Table,
    Id,
    Name,
    MusicbrainzId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Album {
    Table,
    Id,
    Title,
    MusicbrainzId,
    Year,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Track {
    Table,
    Id,
    AlbumId,
    Title,
    #[allow(clippy::enum_variant_names)]
    TrackNumber,
    Duration,
    MusicbrainzId,
    FilePath,
    Sha256,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AlbumArtist {
    Table,
    AlbumId,
    ArtistId,
    IsPrimary,
}

#[derive(DeriveIden)]
enum TrackArtist {
    Table,
    TrackId,
    ArtistId,
    IsPrimary,
}
