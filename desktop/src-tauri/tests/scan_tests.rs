use std::fs;
use std::path::{Path, PathBuf};

use stenodrop_lib::engine::scan::{self, AUDIO_EXTENSIONS};

fn touch(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"x").unwrap();
}

#[test]
fn matches_all_21_extensions_case_insensitively() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(AUDIO_EXTENSIONS.len(), 21, "must match the Mac app's list");
    for (i, ext) in AUDIO_EXTENSIONS.iter().enumerate() {
        // Alternate case to prove the filter is case-insensitive.
        let ext = if i % 2 == 0 { ext.to_uppercase() } else { ext.to_string() };
        touch(&dir.path().join(format!("file{i:02}.{ext}")));
    }
    touch(&dir.path().join("notes.txt"));
    touch(&dir.path().join("no_extension"));

    let found = scan::scan(&[dir.path().to_path_buf()]);
    assert_eq!(found.len(), 21);
}

#[test]
fn recurses_into_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("top.mp3"));
    touch(&dir.path().join("a/b/c/deep.wav"));

    let found = scan::scan(&[dir.path().to_path_buf()]);
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|p| p.ends_with("deep.wav")));
}

#[test]
fn skips_hidden_files_and_hidden_directories() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("visible.mp3"));
    touch(&dir.path().join(".hidden.mp3"));
    touch(&dir.path().join(".git/objects/blob.wav"));

    let found = scan::scan(&[dir.path().to_path_buf()]);
    assert_eq!(found.len(), 1);
    assert!(found[0].ends_with("visible.mp3"));
}

#[test]
fn dedupes_by_canonical_path() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("once.mp3");
    touch(&file);

    // Same directory twice, plus the file listed directly on top.
    let found = scan::scan(&[
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
        file.clone(),
    ]);
    assert_eq!(found.len(), 1);
}

#[test]
fn results_are_sorted() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["zulu.mp3", "alpha.wav", "mike.flac"] {
        touch(&dir.path().join(name));
    }
    let found = scan::scan(&[dir.path().to_path_buf()]);
    let names: Vec<_> = found
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(names, ["alpha.wav", "mike.flac", "zulu.mp3"]);
}

#[test]
fn direct_file_inputs_are_filtered_by_extension() {
    let dir = tempfile::tempdir().unwrap();
    let audio = dir.path().join("keep.m4a");
    let text = dir.path().join("skip.txt");
    let missing: PathBuf = dir.path().join("ghost.mp3");
    touch(&audio);
    touch(&text);

    let found = scan::scan(&[audio.clone(), text, missing]);
    assert_eq!(found.len(), 1);
    assert!(found[0].ends_with("keep.m4a"));
}
