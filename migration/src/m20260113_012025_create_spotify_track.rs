use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create spotify_track table
        manager
            .create_table(
                Table::create()
                    .table("spotify_track")
                    .if_not_exists()
                    .col(
                        ColumnDef::new("spotify_track_id")
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new("title").string().not_null())
                    .col(ColumnDef::new("duration").integer())
                    .col(ColumnDef::new("artists").text().not_null())
                    .col(ColumnDef::new("album").string().not_null())
                    .col(ColumnDef::new("isrc").string())
                    .col(ColumnDef::new("barcode").string())
                    .col(ColumnDef::new("created_at").big_integer().not_null())
                    .col(ColumnDef::new("updated_at").big_integer().not_null())
                    .to_owned(),
            )
            .await?;

        // Create spotify_track_playlist junction table
        manager
            .create_table(
                Table::create()
                    .table("spotify_track_playlist")
                    .if_not_exists()
                    .col(ColumnDef::new("spotify_track_id").big_integer().not_null())
                    .col(
                        ColumnDef::new("spotify_playlist_id")
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col("spotify_track_id")
                            .col("spotify_playlist_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_spotify_track_playlist_track_id")
                            .from("spotify_track_playlist", "spotify_track_id")
                            .to("spotify_track", "spotify_track_id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_spotify_track_playlist_playlist_id")
                            .from("spotify_track_playlist", "spotify_playlist_id")
                            .to("spotify_playlist", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("spotify_track_playlist").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("spotify_track").to_owned())
            .await?;

        Ok(())
    }
}
