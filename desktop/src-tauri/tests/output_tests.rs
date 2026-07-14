use std::collections::HashSet;
use std::path::{Path, PathBuf};

use stenodrop_lib::engine::output::output_path;

fn claimed(paths: &[&str]) -> HashSet<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

#[test]
fn txt_lands_beside_source() {
    let out = output_path(Path::new("/tmp/case/song.mp3"), &HashSet::new());
    assert_eq!(out, PathBuf::from("/tmp/case/song.txt"));
}

#[test]
fn same_basename_collision_falls_back_to_full_name() {
    // a.mp3 already claimed a.txt; a.wav must not clobber it.
    let out = output_path(
        Path::new("/tmp/case/a.wav"),
        &claimed(&["/tmp/case/a.txt"]),
    );
    assert_eq!(out, PathBuf::from("/tmp/case/a.wav.txt"));
}

#[test]
fn collision_only_applies_within_same_directory() {
    let out = output_path(
        Path::new("/tmp/case/sub/a.wav"),
        &claimed(&["/tmp/case/a.txt"]),
    );
    assert_eq!(out, PathBuf::from("/tmp/case/sub/a.txt"));
}

#[test]
fn source_without_extension_gets_txt_appended() {
    let out = output_path(Path::new("/tmp/case/recording"), &HashSet::new());
    assert_eq!(out, PathBuf::from("/tmp/case/recording.txt"));
}

#[test]
fn unicode_names_survive() {
    let out = output_path(Path::new("/tmp/case/récit fumé.mp3"), &HashSet::new());
    assert_eq!(out, PathBuf::from("/tmp/case/récit fumé.txt"));

    let out = output_path(
        Path::new("/tmp/case/اردو نوٹ.mp3"),
        &claimed(&["/tmp/case/اردو نوٹ.txt"]),
    );
    assert_eq!(out, PathBuf::from("/tmp/case/اردو نوٹ.mp3.txt"));
}
