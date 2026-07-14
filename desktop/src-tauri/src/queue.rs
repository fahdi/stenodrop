//! Sequential job queue state machine. Pure logic — tauri event plumbing
//! lives in `commands.rs` so this stays unit-testable.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

use crate::engine::{output, scan};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Transcribing,
    Done,
    DoneWithWarning(String),
    Failed(String),
}

impl JobStatus {
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            JobStatus::Done | JobStatus::DoneWithWarning(_) | JobStatus::Failed(_)
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, JobStatus::Transcribing)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Job {
    pub id: u64,
    pub source: PathBuf,
    pub output: PathBuf,
    pub status: JobStatus,
    pub transcript: String,
}

pub struct QueueState {
    jobs: Vec<Job>,
    next_id: u64,
}

impl Default for QueueState {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueState {
    pub fn new() -> Self {
        Self { jobs: Vec::new(), next_id: 1 }
    }

    /// Scan `inputs` (files/folders), skip sources already pending, assign
    /// collision-safe outputs. Returns how many jobs were added.
    pub fn ingest(&mut self, inputs: &[PathBuf]) -> usize {
        let files = scan::scan(inputs);

        // Unfinished sources may not queue twice; finished ones may re-run.
        let mut seen: HashSet<PathBuf> = self
            .jobs
            .iter()
            .filter(|j| !j.status.is_finished())
            .map(|j| j.source.clone())
            .collect();
        // Outputs claimed by every job still in the list (finished included:
        // their txt exists on disk and must not be clobbered mid-session).
        let mut claimed: HashSet<PathBuf> =
            self.jobs.iter().map(|j| j.output.clone()).collect();

        let mut added = 0;
        for file in files {
            if !seen.insert(file.clone()) {
                continue;
            }
            let output = output::output_path(&file, &claimed);
            claimed.insert(output.clone());
            self.jobs.push(Job {
                id: self.next_id,
                source: file,
                output,
                status: JobStatus::Queued,
                transcript: String::new(),
            });
            self.next_id += 1;
            added += 1;
        }
        added
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }

    /// Move the first Queued job to Transcribing and return a copy.
    /// Returns None while another job is active (sequential queue).
    pub fn start_next(&mut self) -> Option<Job> {
        if self.jobs.iter().any(|j| j.status.is_active()) {
            return None;
        }
        let job = self
            .jobs
            .iter_mut()
            .find(|j| j.status == JobStatus::Queued)?;
        job.status = JobStatus::Transcribing;
        Some(job.clone())
    }

    pub fn finish(&mut self, id: u64, status: JobStatus, transcript: String) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.status = status;
            job.transcript = transcript;
        }
    }

    pub fn clear_finished(&mut self) {
        self.jobs.retain(|j| !j.status.is_finished());
    }

    /// Remove jobs still waiting; active/finished jobs stay. Returns count.
    pub fn cancel_queued(&mut self) -> usize {
        let before = self.jobs.len();
        self.jobs.retain(|j| j.status != JobStatus::Queued);
        before - self.jobs.len()
    }

    /// Queued or actively transcribing — mirrors the Mac app's hasActiveWork.
    pub fn has_active_work(&self) -> bool {
        self.jobs
            .iter()
            .any(|j| j.status == JobStatus::Queued || j.status.is_active())
    }
}

/// Drain the queue sequentially. Each job: transcribe → write txt → finish.
/// A per-file failure marks that job Failed and the queue continues.
/// `on_change` fires after every state transition.
pub fn run_queue<F, E>(state: &Mutex<QueueState>, mut transcribe: F, mut on_change: E)
where
    F: FnMut(&Path) -> Result<String, String>,
    E: FnMut(&[Job]),
{
    loop {
        let job = {
            let mut q = state.lock().unwrap();
            let job = q.start_next();
            if job.is_some() {
                on_change(q.jobs());
            }
            job
        };
        let Some(job) = job else { break };

        let (status, transcript) = match transcribe(&job.source) {
            Ok(text) => match fs::write(&job.output, &text) {
                Ok(()) => (JobStatus::Done, text),
                Err(e) => {
                    let name = job
                        .output
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    (
                        JobStatus::DoneWithWarning(format!("Couldn't save {name}: {e}")),
                        text,
                    )
                }
            },
            Err(e) => (JobStatus::Failed(e), String::new()),
        };

        let mut q = state.lock().unwrap();
        q.finish(job.id, status, transcript);
        on_change(q.jobs());
    }
}
