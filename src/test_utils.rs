use std::sync::Arc;

use sea_orm::{ConnectionTrait, Database as SeaDatabase};

use crate::database::Database;

pub async fn test_db() -> Arc<Database> {
    let conn = SeaDatabase::connect("sqlite::memory:?mode=rwc")
        .await
        .unwrap();

    // Enable foreign keys
    conn.execute_unprepared("PRAGMA foreign_keys = ON")
        .await
        .unwrap();

    let schema = include_str!("../schema.sql");
    for stmt in schema.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            // Strip comment-only lines
            let without_comments: String = trimmed
                .lines()
                .filter(|line| !line.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n");
            let without_comments = without_comments.trim();
            if !without_comments.is_empty() {
                conn.execute_unprepared(without_comments)
                    .await
                    .unwrap_or_else(|e| {
                        panic!(
                            "Failed to execute SQL: {}\nStatement: {}",
                            e, without_comments
                        )
                    });
            }
        }
    }

    Arc::new(Database { conn })
}
