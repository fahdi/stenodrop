//! Folder ingest: recursive walk, audio-extension filter, hidden-file skip,
//! dedupe by canonical path, sorted output. Mirrors the Mac app's
//! `JobQueue.audioExtensions` + `audioFiles(in:)`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The exact 21 extensions the Mac app accepts (JobQueue.swift).
pub const AUDIO_EXTENSIONS: [&str; 21] = [
    "wav", "mp3", "m4a", "m4b", "aac", "flac", "ogg", "oga", "opus", "aiff",
    "aif", "caf", "amr", "wma", "3gp", "mp4", "mov", "m4v", "avi", "webm",
    "mkv",
];

/// True when the path has one of the supported extensions (case-insensitive).
pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| {
            let e = e.to_ascii_lowercase();
            AUDIO_EXTENSIONS.contains(&e.as_str())
        })
}

/// Expand a mixed list of files and directories into a deduped, sorted list
/// of audio files. Hidden files/directories are skipped during recursion;
/// explicitly listed files only need to pass the extension filter (same
/// semantics as the Mac drop handler).
pub fn scan(inputs: &[PathBuf]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut found = Vec::new();

    for input in inputs {
        let Ok(meta) = fs::metadata(input) else {
            continue; // vanished or unreadable — skip silently, like the Mac app
        };
        if meta.is_dir() {
            walk(input, &mut seen, &mut found);
        } else if meta.is_file() && is_audio_file(input) {
            push_unique(input, &mut seen, &mut found);
        }
    }

    found.sort();
    found
}

fn walk(dir: &Path, seen: &mut HashSet<PathBuf>, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue; // hidden file or directory
        }
        let Ok(file_type) = entry.file_type() else { continue };
        let path = entry.path();
        if file_type.is_dir() {
            walk(&path, seen, found);
        } else if is_audio_file(&path) {
            push_unique(&path, seen, found);
        }
    }
}

fn push_unique(path: &Path, seen: &mut HashSet<PathBuf>, found: &mut Vec<PathBuf>) {
    // Canonicalize so the same file reached via different spellings
    // (symlinks, `..`, repeated inputs) queues only once.
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if seen.insert(canonical.clone()) {
        found.push(canonical);
    }
}
