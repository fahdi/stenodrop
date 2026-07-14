//! Real-model integration test. Run explicitly with:
//!   cargo test --test whisper_integration -- --ignored --nocapture
//! Requires ggml-small.bin in the per-OS app-data dir
//! (macOS: ~/Library/Application Support/StenoDrop/models/).

use std::path::PathBuf;

use stenodrop_lib::engine::decode::decode_to_mono_16k;
use stenodrop_lib::engine::whisper::Transcriber;
use stenodrop_lib::model;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
#[ignore = "needs the 466 MB ggml-small model; run with -- --ignored"]
fn transcribes_jfk_fixture_containing_country() {
    let model = model::model_path();
    assert!(
        model.exists(),
        "model not found at {} — download it via the app first",
        model.display()
    );
    assert!(model::model_is_ready(), "model file is truncated");

    let samples = decode_to_mono_16k(&fixture("jfk.wav")).expect("decode fixture");
    let engine = Transcriber::load(&model).expect("load model");

    let text = engine
        .transcribe(&samples, false, "en")
        .expect("transcription");
    println!("TRANSCRIPT: {text}");
    assert!(
        text.to_lowercase().contains("country"),
        "expected 'country' in transcript, got: {text}"
    );

    // Translate path must also run (English in → English out is a no-op,
    // but exercises the translate flag end-to-end).
    let translated = engine
        .transcribe(&samples, true, "auto")
        .expect("translate run");
    assert!(!translated.trim().is_empty());
}
