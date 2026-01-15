use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Track {
    #[allow(dead_code)]
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
enum Album {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum TrackArtist {
    Table,
    TrackId,
    ArtistId,
    IsPrimary,
}

#[derive(DeriveIden)]
enum Artist {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Create new tracks table with same schema as track
        manager
            .create_table(
                Table::create()
                    .table("tracks")
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
                            .from("tracks", Track::AlbumId)
                            .to(Album::Table, Album::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Step 2: Copy all data from track to tracks
        let db = manager.get_connection();
        db.execute_unprepared("INSERT INTO tracks SELECT * FROM track")
            .await?;

        // Step 3: Save track_artist data to temporary table
        db.execute_unprepared(
            "CREATE TEMP TABLE track_artist_backup AS SELECT * FROM track_artist",
        )
        .await?;

        // Step 4: Drop track_artist table (it has foreign key to track)
        manager
            .drop_table(Table::drop().table(TrackArtist::Table).to_owned())
            .await?;

        // Step 5: Recreate track_artist with foreign key to tracks
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
                            .to("tracks", Track::Id)
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

        // Step 6: Restore track_artist data from backup
        db.execute_unprepared("INSERT INTO track_artist SELECT * FROM track_artist_backup")
            .await?;

        // Step 7: Drop old track table
        manager
            .drop_table(Table::drop().table("track").to_owned())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse the process: rename tracks back to track

        // Step 1: Create track table
        manager
            .create_table(
                Table::create()
                    .table("track")
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
                            .from("track", Track::AlbumId)
                            .to(Album::Table, Album::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Step 2: Copy data from tracks to track
        let db = manager.get_connection();
        db.execute_unprepared("INSERT INTO track SELECT * FROM tracks")
            .await?;

        // Step 3: Save track_artist data to temporary table
        db.execute_unprepared(
            "CREATE TEMP TABLE track_artist_backup AS SELECT * FROM track_artist",
        )
        .await?;

        // Step 4: Drop track_artist table
        manager
            .drop_table(Table::drop().table(TrackArtist::Table).to_owned())
            .await?;

        // Step 5: Recreate track_artist with foreign key to track
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
                            .to("track", Track::Id)
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

        // Step 6: Restore track_artist data from backup
        db.execute_unprepared("INSERT INTO track_artist SELECT * FROM track_artist_backup")
            .await?;

        // Step 7: Drop tracks table
        manager
            .drop_table(Table::drop().table("tracks").to_owned())
            .await?;

        Ok(())
    }
}
