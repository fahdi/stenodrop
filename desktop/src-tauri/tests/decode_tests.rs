use std::path::PathBuf;

use stenodrop_lib::engine::decode::{decode_to_mono_16k, TARGET_SAMPLE_RATE};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Fixtures are 0.5 s of 440 Hz sine at 44.1 kHz stereo. After decode we
/// expect ~8000 mono samples at 16 kHz; lossy codecs pad the edges, so
/// allow a generous window.
fn assert_tone(samples: &[f32]) {
    let expected = TARGET_SAMPLE_RATE as usize / 2;
    let (lo, hi) = (expected * 3 / 4, expected * 7 / 4);
    assert!(
        (lo..hi).contains(&samples.len()),
        "expected ~{expected} samples, got {}",
        samples.len()
    );
    let peak = samples.iter().fold(0f32, |m, s| m.max(s.abs()));
    assert!(peak > 0.1, "decoded audio is near-silent (peak {peak})");
    assert!(peak <= 1.5, "samples not normalized (peak {peak})");
}

#[test]
fn wav_decodes_to_16k_mono() {
    assert_tone(&decode_to_mono_16k(&fixture("tone.wav")).unwrap());
}

#[test]
fn mp3_decodes_to_16k_mono() {
    assert_tone(&decode_to_mono_16k(&fixture("tone.mp3")).unwrap());
}

#[test]
fn flac_decodes_to_16k_mono() {
    assert_tone(&decode_to_mono_16k(&fixture("tone.flac")).unwrap());
}

#[test]
fn sixteen_k_speech_wav_passes_through() {
    // jfk.wav is already 16 kHz mono; must decode without resample damage.
    let samples = decode_to_mono_16k(&fixture("jfk.wav")).unwrap();
    assert!(samples.len() > TARGET_SAMPLE_RATE as usize); // > 1 s of speech
}

#[test]
fn non_audio_input_is_an_error() {
    assert!(decode_to_mono_16k(&fixture("not_audio.txt")).is_err());
}

#[test]
fn missing_file_is_an_error() {
    assert!(decode_to_mono_16k(&fixture("does_not_exist.wav")).is_err());
}
