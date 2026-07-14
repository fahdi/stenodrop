//! Tauri commands + the worker thread that drains the queue and streams
//! status events to the UI.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::engine::{decode, whisper::Transcriber};
use crate::model;
use crate::queue::{run_queue, Job, JobStatus, QueueState};

pub struct EngineSettings {
    pub translate: bool,
    pub language: String,
}

impl Default for EngineSettings {
    fn default() -> Self {
        // Same defaults as the Mac app: translate ON, auto-detect.
        Self { translate: true, language: "auto".into() }
    }
}

#[derive(Default)]
pub struct AppState {
    pub queue: Arc<Mutex<QueueState>>,
    pub settings: Arc<Mutex<EngineSettings>>,
    pub worker_running: Arc<AtomicBool>,
    pub downloading: Arc<AtomicBool>,
}

fn emit_queue(app: &AppHandle, jobs: &[Job]) {
    let _ = app.emit("queue-changed", jobs);
}

// ---------- queue commands ----------

#[tauri::command]
pub fn get_jobs(state: State<AppState>) -> Vec<Job> {
    state.queue.lock().unwrap().jobs().to_vec()
}

#[tauri::command]
pub fn ingest_paths(app: AppHandle, state: State<'_, AppState>, paths: Vec<String>) -> usize {
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let added = {
        let mut q = state.queue.lock().unwrap();
        let added = q.ingest(&inputs);
        emit_queue(&app, q.jobs());
        added
    };
    if added > 0 {
        ensure_worker(app);
    }
    added
}

#[tauri::command]
pub fn set_settings(state: State<AppState>, translate: bool, language: String) {
    let mut s = state.settings.lock().unwrap();
    s.translate = translate;
    s.language = language;
}

#[tauri::command]
pub fn clear_finished(app: AppHandle, state: State<AppState>) {
    let mut q = state.queue.lock().unwrap();
    q.clear_finished();
    emit_queue(&app, q.jobs());
}

#[tauri::command]
pub fn cancel_queued(app: AppHandle, state: State<AppState>) -> usize {
    let mut q = state.queue.lock().unwrap();
    let removed = q.cancel_queued();
    emit_queue(&app, q.jobs());
    removed
}

#[tauri::command]
pub fn has_active_work(state: State<AppState>) -> bool {
    state.queue.lock().unwrap().has_active_work()
}

// ---------- worker ----------

/// Spawn the sequential transcription worker if it isn't already running.
pub fn ensure_worker(app: AppHandle) {
    let state = app.state::<AppState>();
    if state.worker_running.swap(true, Ordering::SeqCst) {
        return; // already draining
    }
    let queue = state.queue.clone();
    let settings = state.settings.clone();
    let running = state.worker_running.clone();

    std::thread::spawn(move || loop {
        // Model loads once per drain, not once per file.
        let mut transcriber: Option<Transcriber> = None;
        let events_app = app.clone();

        run_queue(
            &queue,
            |source| {
                let (translate, language) = {
                    let s = settings.lock().unwrap();
                    (s.translate, s.language.clone())
                };
                let samples =
                    decode::decode_to_mono_16k(source).map_err(|e| e.to_string())?;
                if transcriber.is_none() {
                    if !model::model_is_ready() {
                        return Err("Whisper model not downloaded yet.".into());
                    }
                    transcriber = Some(Transcriber::load(&model::model_path())?);
                }
                transcriber
                    .as_ref()
                    .unwrap()
                    .transcribe(&samples, translate, &language)
            },
            move |jobs| emit_queue(&events_app, jobs),
        );

        running.store(false, Ordering::SeqCst);
        // Close the race: a drop may have landed between the drain ending
        // and the flag clearing. If so — and no other worker claimed the
        // flag — keep going.
        let has_queued = queue
            .lock()
            .unwrap()
            .jobs()
            .iter()
            .any(|j| j.status == JobStatus::Queued);
        if has_queued && !running.swap(true, Ordering::SeqCst) {
            continue;
        }
        break;
    });
}

// ---------- model commands ----------

#[tauri::command]
pub fn model_ready() -> bool {
    model::model_is_ready()
}

#[derive(Clone, Serialize)]
struct ModelProgress {
    fraction: f64,
    downloaded: u64,
    total: Option<u64>,
    done: bool,
    error: Option<String>,
}

#[tauri::command]
pub fn download_model(app: AppHandle, state: State<AppState>) {
    if state.downloading.swap(true, Ordering::SeqCst) {
        return; // one download at a time
    }
    let downloading = state.downloading.clone();

    std::thread::spawn(move || {
        let mut last_permille = 0u64;
        let result = model::download(&mut |downloaded, total| {
            // Throttle IPC: emit at most once per 0.1 % step.
            let permille = total
                .map(|t| downloaded.saturating_mul(1000) / t.max(1))
                .unwrap_or(0);
            if permille == last_permille && downloaded != total.unwrap_or(0) {
                return;
            }
            last_permille = permille;
            let fraction = total
                .map(|t| downloaded as f64 / t as f64)
                .unwrap_or(0.0);
            let _ = app.emit(
                "model-progress",
                ModelProgress { fraction, downloaded, total, done: false, error: None },
            );
        });

        let payload = match result {
            Ok(()) => ModelProgress {
                fraction: 1.0,
                downloaded: 0,
                total: None,
                done: true,
                error: None,
            },
            Err(message) => ModelProgress {
                fraction: 0.0,
                downloaded: 0,
                total: None,
                done: false,
                error: Some(message),
            },
        };
        let _ = app.emit("model-progress", payload);
        downloading.store(false, Ordering::SeqCst);
    });
}
