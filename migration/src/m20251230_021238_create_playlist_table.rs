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
                    .table("playlists")
                    .if_not_exists()
                    .col(
                        ColumnDef::new("id")
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new("name").string().not_null())
                    .col(ColumnDef::new("description").string())
                    .col(ColumnDef::new("created_at").timestamp().not_null())
                    .col(ColumnDef::new("updated_at").timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // Create playlist_tracks junction table
        manager
            .create_table(
                Table::create()
                    .table("playlist_tracks")
                    .if_not_exists()
                    .col(ColumnDef::new("playlist_id").integer().not_null())
                    .col(ColumnDef::new("track_id").integer().not_null())
                    .col(ColumnDef::new("created_at").timestamp().not_null())
                    .col(ColumnDef::new("updated_at").timestamp().not_null())
                    .primary_key(Index::create().col("playlist_id").col("track_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_tracks_playlist_id")
                            .from("playlist_tracks", "playlist_id")
                            .to("playlists", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_tracks_track_id")
                            .from("playlist_tracks", "track_id")
                            .to("tracks", "id")
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
            .drop_table(Table::drop().table("playlist_tracks").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table("playlists").to_owned())
            .await?;

        Ok(())
    }
}
