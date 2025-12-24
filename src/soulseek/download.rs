// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use std::path::Path;
use std::sync::mpsc;

use color_eyre::{Result, eyre::Context};

use crate::soulseek::client::SoulSeekClientContext;
use crate::soulseek::types::SingleFileResult;

// ============================================================================
// Download Function
// ============================================================================

pub async fn download_file(
    result: &SingleFileResult,
    download_folder: &Path,
    context: &SoulSeekClientContext,
) -> Result<mpsc::Receiver<soulseek_rs::DownloadStatus>> {
    log::debug!(
        "Starting download: '{}' from user '{}' ({} bytes)",
        result.filename,
        result.username,
        result.size
    );

    // Ensure download directory exists
    tokio::fs::create_dir_all(download_folder).await?;

    // Get the soulseek client for direct download access
    let client = context
        .soulseek_client
        .as_ref()
        .ok_or_else(|| color_eyre::eyre::eyre!("SoulSeek client not available for download"))?
        .clone();

    let filename = result.filename.clone();
    let username = result.username.clone();
    let size = result.size;

    let receiver = client
        .download(
            filename.clone(),
            username.clone(),
            size,
            download_folder.as_os_str().to_str().unwrap().to_string(),
        )
        .context("Failed to download file")?;

    log::info!("Download initiated: '{}' from '{}'", filename, username);
    Ok(receiver)
}

