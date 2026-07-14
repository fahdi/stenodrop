//! ggml-small model management: location, readiness, validated download.
//! Mirrors the Mac app's ModelDownloader fixes: HTTP status and byte-size
//! are both validated before the file is moved into place.

use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin";

/// ggml-small.bin is ~466 MB; anything smaller is an error page or a
/// truncated body, even if the HTTP layer called it a success.
pub const MIN_VALID_SIZE: u64 = 400_000_000;

/// Per-OS app-data dir: macOS `~/Library/Application Support/StenoDrop/models`,
/// Windows `%APPDATA%\StenoDrop\models`, Linux `~/.local/share/StenoDrop/models`.
pub fn models_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("StenoDrop")
        .join("models")
}

pub fn model_path() -> PathBuf {
    models_dir().join("ggml-small.bin")
}

pub fn model_is_ready() -> bool {
    fs::metadata(model_path())
        .map(|m| m.len() > MIN_VALID_SIZE)
        .unwrap_or(false)
}

/// Pure validation shared by download and tests.
pub fn validate_download(http_status: u16, size: u64) -> Result<(), String> {
    if http_status != 200 {
        return Err(format!("Download failed (HTTP {http_status}). Try again."));
    }
    if size < MIN_VALID_SIZE {
        return Err(format!(
            "Download incomplete ({} MB of ~466 MB). Check your connection and try again.",
            size / 1_000_000
        ));
    }
    Ok(())
}

/// Download the model to `models_dir()`, reporting (downloaded, total) bytes.
/// The body is streamed to a `.partial` file and only renamed into place
/// after `validate_download` passes — a 404 body or captive-portal page can
/// never masquerade as a model.
pub fn download(progress: &mut dyn FnMut(u64, Option<u64>)) -> Result<(), String> {
    fs::create_dir_all(models_dir())
        .map_err(|e| format!("could not create models folder: {e}"))?;

    let response = ureq::get(MODEL_URL).call().map_err(|e| match e {
        ureq::Error::Status(code, _) => format!("Download failed (HTTP {code}). Try again."),
        other => format!("Download failed: {other}"),
    })?;
    let status = response.status();
    let total = response
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok());

    let partial = models_dir().join("ggml-small.bin.partial");
    let mut reader = response.into_reader();
    let mut file = fs::File::create(&partial)
        .map_err(|e| format!("could not create {}: {e}", partial.display()))?;

    let mut buf = [0u8; 128 * 1024];
    let mut downloaded: u64 = 0;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Download interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("could not write model file: {e}"))?;
        downloaded += n as u64;
        progress(downloaded, total);
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    if let Err(e) = validate_download(status, downloaded) {
        let _ = fs::remove_file(&partial);
        return Err(e);
    }

    let dest = model_path();
    let _ = fs::remove_file(&dest);
    fs::rename(&partial, &dest).map_err(|e| format!("could not install model: {e}"))?;
    Ok(())
}
