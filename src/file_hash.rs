use color_eyre::{Result, eyre::Context};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing;

/// Compute the SHA-256 hash of a file
pub fn compute_sha256(path: &Path) -> Result<String> {
    tracing::debug!("Computing SHA-256 hash for: {}", path.display());

    let mut file = File::open(path).context(format!("Failed to open file: {}", path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192]; // 8KB buffer for efficient reading

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .context(format!("Failed to read file: {}", path.display()))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);
    tracing::info!("Hash computed: {}", hash_str);
    Ok(hash_str)
}
