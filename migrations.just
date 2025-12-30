sqlite_database := "sqlite:./music/library.db"

migrate-down:
    sea-orm-cli migrate down -u {{sqlite_database}}

migrate-up:
    sea-orm-cli migrate up -u {{sqlite_database}}

migrate-generate name:
    sea-orm-cli migrate generate "{{name}}"