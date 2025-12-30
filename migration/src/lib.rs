pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_tables;
mod m20251216_050017_unimportable_file;
mod m20251230_021238_create_playlist_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tables::Migration),
            Box::new(m20251216_050017_unimportable_file::Migration),
            Box::new(m20251230_021238_create_playlist_table::Migration),
        ]
    }
}
