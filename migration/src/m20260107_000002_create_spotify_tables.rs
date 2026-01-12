use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create spotify_accounts table
        manager
            .create_table(
                Table::create()
                    .table(SpotifyAccount::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SpotifyAccount::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccount::UserId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(SpotifyAccount::DisplayName).string())
                    .col(
                        ColumnDef::new(SpotifyAccount::AccessToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccount::RefreshToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccount::TokenExpiry)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccount::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccount::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create spotify_playlists table
        manager
            .create_table(
                Table::create()
                    .table(SpotifyPlaylist::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SpotifyPlaylist::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylist::AccountId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylist::SpotifyId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(SpotifyPlaylist::Name).string().not_null())
                    .col(ColumnDef::new(SpotifyPlaylist::Description).string())
                    .col(
                        ColumnDef::new(SpotifyPlaylist::SnapshotId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylist::TrackCount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylist::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylist::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_spotify_playlists_account_id")
                            .from(SpotifyPlaylist::Table, SpotifyPlaylist::AccountId)
                            .to(SpotifyAccount::Table, SpotifyAccount::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create spotify_playlist_sync_state table
        manager
            .create_table(
                Table::create()
                    .table(SpotifyPlaylistSyncState::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SpotifyPlaylistSyncState::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylistSyncState::SpotifyPlaylistId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SpotifyPlaylistSyncState::LocalPlaylistId).big_integer())
                    .col(ColumnDef::new(SpotifyPlaylistSyncState::LastSyncAt).big_integer())
                    .col(
                        ColumnDef::new(SpotifyPlaylistSyncState::SyncStatus)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylistSyncState::TracksDownloaded)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylistSyncState::TracksFailed)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(SpotifyPlaylistSyncState::ErrorLog).text())
                    .col(
                        ColumnDef::new(SpotifyPlaylistSyncState::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyPlaylistSyncState::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_spotify_playlist_sync_state_spotify_playlist_id")
                            .from(
                                SpotifyPlaylistSyncState::Table,
                                SpotifyPlaylistSyncState::SpotifyPlaylistId,
                            )
                            .to(SpotifyPlaylist::Table, SpotifyPlaylist::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_spotify_playlist_sync_state_local_playlist_id")
                            .from(
                                SpotifyPlaylistSyncState::Table,
                                SpotifyPlaylistSyncState::LocalPlaylistId,
                            )
                            .to("playlists", "id")
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create spotify_track_download_failures table
        manager
            .create_table(
                Table::create()
                    .table(SpotifyTrackDownloadFailure::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::SpotifyPlaylistId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::SpotifyTrackId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::TrackName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::ArtistName)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SpotifyTrackDownloadFailure::AlbumName).string())
                    .col(ColumnDef::new(SpotifyTrackDownloadFailure::Isrc).string())
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::Reason)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::AttemptsCount)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyTrackDownloadFailure::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_spotify_track_download_failure_playlist_id")
                            .from(
                                SpotifyTrackDownloadFailure::Table,
                                SpotifyTrackDownloadFailure::SpotifyPlaylistId,
                            )
                            .to(SpotifyPlaylist::Table, SpotifyPlaylist::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_spotify_playlists_account_id")
                    .table(SpotifyPlaylist::Table)
                    .col(SpotifyPlaylist::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_spotify_playlist_sync_state_spotify_playlist_id")
                    .table(SpotifyPlaylistSyncState::Table)
                    .col(SpotifyPlaylistSyncState::SpotifyPlaylistId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SpotifyTrackDownloadFailure::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(SpotifyPlaylistSyncState::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(SpotifyPlaylist::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SpotifyAccount::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum SpotifyAccount {
    Table,
    Id,
    UserId,
    DisplayName,
    AccessToken,
    RefreshToken,
    TokenExpiry,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SpotifyPlaylist {
    Table,
    Id,
    AccountId,
    SpotifyId,
    Name,
    Description,
    SnapshotId,
    TrackCount,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SpotifyPlaylistSyncState {
    Table,
    Id,
    SpotifyPlaylistId,
    LocalPlaylistId,
    LastSyncAt,
    SyncStatus,
    TracksDownloaded,
    TracksFailed,
    ErrorLog,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SpotifyTrackDownloadFailure {
    Table,
    Id,
    SpotifyPlaylistId,
    SpotifyTrackId,
    TrackName,
    ArtistName,
    AlbumName,
    Isrc,
    Reason,
    AttemptsCount,
    CreatedAt,
    UpdatedAt,
}
