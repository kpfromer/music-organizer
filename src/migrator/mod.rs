use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Failed to find migrations directory")]
    FailedToFindMigrationsDirectory,
    #[error("Failed to find database path: {0}")]
    FailedToFindDatabasePath(PathBuf),
    #[error(
        "Atlas command not found. Please install Atlas CLI: https://atlasgo.io/cli/installation"
    )]
    AtlasCommandNotFound,
    #[error("Failed to execute atlas process:\nstdout:\n{stdout}\nstderr:\n{stderr}")]
    AtlasCommandFailed { stdout: String, stderr: String },
    #[error("Unknown error: {0}")]
    UnknownError(#[from] std::io::Error),
}

/// Runs all pending migrations against the given database path.
/// Returns an error if the migrations directory or database path cannot be found, or if the atlas command fails.
pub fn run_migrations(database_path: &Path) -> Result<(), MigrationError> {
    let migrations_path = Path::new("./migrations")
        .canonicalize()
        .map_err(|_| MigrationError::FailedToFindMigrationsDirectory)?;
    if !migrations_path.is_dir() {
        return Err(MigrationError::FailedToFindMigrationsDirectory);
    }
    let migrations_path_str = migrations_path
        .to_str()
        .ok_or(MigrationError::FailedToFindMigrationsDirectory)?;
    let migrations_option = format!("file://{}", migrations_path_str);

    // Canonicalize parent directory (must exist) and append filename
    // This allows the database file to not exist yet (Atlas/SQLite will create it)
    let db_parent = database_path
        .parent()
        .ok_or_else(|| MigrationError::FailedToFindDatabasePath(database_path.to_path_buf()))?
        .canonicalize()
        .map_err(|_| MigrationError::FailedToFindDatabasePath(database_path.to_path_buf()))?;
    let db_filename = database_path
        .file_name()
        .ok_or_else(|| MigrationError::FailedToFindDatabasePath(database_path.to_path_buf()))?;
    let database_path = db_parent
        .join(db_filename)
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| MigrationError::FailedToFindDatabasePath(database_path.to_path_buf()))?;
    let database_option = format!("sqlite://{}", database_path);

    let args = [
        "migrate",
        "apply",
        "--dir",
        &migrations_option,
        "--url",
        &database_option,
    ];

    log::info!("Running migrations `atlas` with arguments: {:?}", args);

    // By default, atlas migrate apply executes all pending migration files.
    // atlas migrate apply
    match Command::new("atlas").args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(MigrationError::AtlasCommandFailed {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                })
            }
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Err(MigrationError::AtlasCommandNotFound)
            } else {
                Err(MigrationError::UnknownError(e))
            }
        }
    }
}
