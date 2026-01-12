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
                    .table("tracks")
                    .add_column(ColumnDef::new("isrcs").string().null())
                    .to_owned(),
            )
            .await?;

        // Add EAN/UPC barcode field
        manager
            .alter_table(
                Table::alter()
                    .table("tracks")
                    .add_column(ColumnDef::new("barcode").string().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("tracks")
                    .drop_column("isrcs")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("tracks")
                    .drop_column("barcode")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
