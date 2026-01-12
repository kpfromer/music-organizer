use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add ISRC field (stores JSON array of ISRCs)
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .add_column(ColumnDef::new(Track::Isrcs).string())
                    .to_owned(),
            )
            .await?;

        // Add EAN/UPC barcode field
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .add_column(ColumnDef::new(Track::Barcode).string())
                    .to_owned(),
            )
            .await?;

        // Create index on ISRC for faster matching
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tracks_isrcs")
                    .table(Track::Table)
                    .col(Track::Isrcs)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_tracks_isrcs").to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .drop_column(Track::Isrcs)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .drop_column(Track::Barcode)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Track {
    Table,
    Isrcs,
    Barcode,
}
