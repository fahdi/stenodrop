//! whisper-rs wrapper: model load once, transcribe many.

use std::path::Path;

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
};

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    /// Load a ggml model from disk. Expensive (~seconds); keep the instance
    /// alive for a whole queue run.
    pub fn load(model: &Path) -> Result<Self, String> {
        let path = model
            .to_str()
            .ok_or_else(|| "model path is not valid UTF-8".to_string())?;
        let ctx = WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .map_err(|e| format!("failed to load whisper model: {e}"))?;
        Ok(Self { ctx })
    }

    /// Transcribe 16 kHz mono f32 samples.
    /// `language` is an ISO-639-1 code or "auto"; `translate` forces
    /// whisper's translate-to-English task.
    pub fn transcribe(
        &self,
        samples: &[f32],
        translate: bool,
        language: &str,
    ) -> Result<String, String> {
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("whisper state: {e}"))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_translate(translate);
        let language = if language.is_empty() { "auto" } else { language };
        params.set_language(Some(language));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        let threads = std::thread::available_parallelism()
            .map(|n| n.get().min(8) as i32)
            .unwrap_or(4);
        params.set_n_threads(threads);

        state
            .full(params, samples)
            .map_err(|e| format!("transcription failed: {e}"))?;

        let mut text = String::new();
        for segment in state.as_iter() {
            match segment.to_str_lossy() {
                Ok(chunk) => text.push_str(&chunk),
                Err(e) => return Err(format!("could not read segment text: {e}")),
            }
        }

        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("transcription produced no text".into());
        }
        Ok(trimmed.to_string())
    }
}
