//! Embeds versioned `.sql` migration files and applies them transactionally
//! at runtime. The Atlas binary is a dev-time tool only and is never invoked
//! at runtime — see TDD 11 §Migrations.
