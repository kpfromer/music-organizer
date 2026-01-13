sqlite_database := "sqlite:./music/library.db"

migrate-down:
    sea-orm-cli migrate down -u {{sqlite_database}}

migrate-up:
    sea-orm-cli migrate up -u {{sqlite_database}}

migrate-generate name:
    sea-orm-cli migrate generate "{{name}}"

create-migration name:
    sea-orm-cli migrate generate "{{name}}"

generate-entity table:
    #!/usr/bin/env sh
    # See https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/
    cp ./src/entities/mod.rs ./src/entities/mod.rs.bak
    sea-orm-cli generate entity \
        --database-url {{sqlite_database}} \
        --with-prelude none \
        --output-dir ./src/entities \
        --entity-format dense \
        --big-integer-type i64 \
        --impl-active-model-behavior \
        --tables "{{table}}"
    mv ./src/entities/mod.rs.bak ./src/entities/mod.rs
