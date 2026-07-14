//! Transcript naming: `song.txt` beside the source, unless another queued
//! source (song.wav vs song.mp3) already claims it — then `song.mp3.txt`.

use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Pick the output path for `source`, avoiding paths in `claimed`.
pub fn output_path(source: &Path, claimed: &HashSet<PathBuf>) -> PathBuf {
    let primary = source.with_extension("txt");
    if !claimed.contains(&primary) {
        return primary;
    }
    // Collision fallback: keep the full source name, append .txt
    // (a.wav → a.wav.txt). Byte-safe for any unicode filename.
    let mut name = source
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("transcript"));
    name.push(".txt");
    source.with_file_name(name)
}
