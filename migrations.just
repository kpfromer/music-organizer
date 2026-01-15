sqlite_database := "sqlite://music/library.db"
migrations_dir := "file://migrations"
schema_file := "file://schema.sql"

# Uses Atlas to sync the database schema with the schema.hcl file for development purposes.
dev-migrate-sync:
    atlas schema apply --url "{{sqlite_database}}" --to "{{schema_file}}" --dev-url "sqlite://dev?mode=memory"

# Creates a new migration file with the given name using Atlas.
create-migration name:
    atlas migrate diff "{{name}}" --dir "{{migrations_dir}}" --to "{{schema_file}}" --dev-url "sqlite://dev?mode=memory"
