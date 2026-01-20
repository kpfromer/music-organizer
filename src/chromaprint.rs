use color_eyre::Result;
use std::path::Path;
use std::process::Command;
use tracing;

pub fn chromaprint_from_file(path: &Path) -> Result<(String, u32)> {
    tracing::debug!("Computing chromaprint fingerprint for: {}", path.display());

    if which::which("fpcalc").is_err() {
        tracing::error!("fpcalc not found in PATH");
        return Err(color_eyre::eyre::eyre!(
            "fpcalc not found in PATH. Please install Chromaprint (fpcalc) and ensure it's available."
        ));
    }

    let output = Command::new("fpcalc").arg(path).output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let mut fingerprint = String::new();
    let mut duration = 0;

    for line in stdout.lines() {
        if let Some(v) = line.strip_prefix("FINGERPRINT=") {
            fingerprint = v.to_string();
        } else if let Some(v) = line.strip_prefix("DURATION=") {
            duration = v.parse()?;
        }
    }

    tracing::info!("Fingerprint computed: duration={}s", duration);
    Ok((fingerprint, duration))
}
