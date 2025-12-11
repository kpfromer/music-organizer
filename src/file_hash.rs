use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Compute the SHA-256 hash of a file
pub fn compute_sha256(path: &Path) -> Result<String> {
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
    Ok(format!("{:x}", hash))
}
