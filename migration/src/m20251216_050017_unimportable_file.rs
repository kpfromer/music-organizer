use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("unimportable_files")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("file_path").not_null())
                    .col(string("sha256").not_null())
                    .col(string("reason").not_null())
                    .col(integer("created_at").not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("unimportable_files").to_owned())
            .await
    }
}
