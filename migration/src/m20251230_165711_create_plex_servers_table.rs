use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("plex_servers")
                    .if_not_exists()
                    .col(
                        ColumnDef::new("id")
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new("name")
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new("server_url")
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new("access_token").string())
                    .col(ColumnDef::new("created_at").timestamp().not_null())
                    .col(ColumnDef::new("updated_at").timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("plex_servers").to_owned())
            .await?;

        Ok(())
    }
}

