use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use stenodrop_lib::queue::{run_queue, JobStatus, QueueState};

fn touch(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"x").unwrap();
}

fn make_files(dir: &Path, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|n| {
            let p = dir.join(n);
            touch(&p);
            p
        })
        .collect()
}

#[test]
fn ingest_scans_and_assigns_collision_safe_outputs() {
    let dir = tempfile::tempdir().unwrap();
    make_files(dir.path(), &["a.mp3", "a.wav", "b.flac"]);

    let mut q = QueueState::new();
    let added = q.ingest(&[dir.path().to_path_buf()]);
    assert_eq!(added, 3);

    let outputs: Vec<String> = q
        .jobs()
        .iter()
        .map(|j| j.output.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    // Sorted scan order: a.mp3 first claims a.txt, a.wav falls back.
    assert!(outputs.contains(&"a.txt".to_string()));
    assert!(outputs.contains(&"a.wav.txt".to_string()));
    assert!(outputs.contains(&"b.txt".to_string()));
}

#[test]
fn ingest_dedupes_pending_but_allows_requeue_after_finish() {
    let dir = tempfile::tempdir().unwrap();
    let files = make_files(dir.path(), &["one.mp3"]);

    let mut q = QueueState::new();
    assert_eq!(q.ingest(&files), 1);
    assert_eq!(q.ingest(&files), 0, "pending duplicate must be skipped");

    let job = q.start_next().unwrap();
    q.finish(job.id, JobStatus::Done, "text".into());

    assert_eq!(q.ingest(&files), 1, "finished file may be queued again");
    assert_eq!(q.jobs().len(), 2);
}

#[test]
fn start_next_transitions_to_transcribing_and_serializes_work() {
    let dir = tempfile::tempdir().unwrap();
    let files = make_files(dir.path(), &["a.mp3", "b.mp3"]);

    let mut q = QueueState::new();
    q.ingest(&files);

    let first = q.start_next().unwrap();
    assert_eq!(q.jobs()[0].status, JobStatus::Transcribing);
    assert!(
        q.start_next().is_none(),
        "no second job may start while one is active"
    );

    q.finish(first.id, JobStatus::Done, String::new());
    let second = q.start_next().unwrap();
    assert_ne!(second.id, first.id);
}

#[test]
fn per_file_failure_continues_the_queue() {
    let dir = tempfile::tempdir().unwrap();
    let files = make_files(dir.path(), &["a.mp3", "boom.mp3", "c.mp3"]);

    let mut q = QueueState::new();
    q.ingest(&files);
    let state = Mutex::new(q);

    run_queue(
        &state,
        |src| {
            if src.file_name().unwrap() == "boom.mp3" {
                Err("decode blew up".into())
            } else {
                Ok(format!("transcript of {}", src.display()))
            }
        },
        |_jobs| {},
    );

    let q = state.into_inner().unwrap();
    let statuses: Vec<&JobStatus> = q.jobs().iter().map(|j| &j.status).collect();
    assert_eq!(statuses[0], &JobStatus::Done);
    assert_eq!(
        statuses[1],
        &JobStatus::Failed("decode blew up".to_string())
    );
    assert_eq!(statuses[2], &JobStatus::Done);

    // Transcripts really landed beside the sources for the successful two.
    assert!(dir.path().join("a.txt").exists());
    assert!(!dir.path().join("boom.txt").exists());
    assert!(dir.path().join("c.txt").exists());
}

#[test]
fn unwritable_output_yields_done_with_warning() {
    let dir = tempfile::tempdir().unwrap();
    let files = make_files(dir.path(), &["a.mp3"]);

    let mut q = QueueState::new();
    q.ingest(&files);
    let state = Mutex::new(q);

    // Remove the whole directory so the txt write must fail.
    fs::remove_dir_all(dir.path()).unwrap();

    run_queue(&state, |_| Ok("text".into()), |_jobs| {});

    let q = state.into_inner().unwrap();
    match &q.jobs()[0].status {
        JobStatus::DoneWithWarning(msg) => assert!(!msg.is_empty()),
        other => panic!("expected DoneWithWarning, got {other:?}"),
    }
    assert_eq!(q.jobs()[0].transcript, "text", "transcript kept in memory");
}

#[test]
fn run_queue_emits_change_events() {
    let dir = tempfile::tempdir().unwrap();
    let files = make_files(dir.path(), &["a.mp3", "b.mp3"]);

    let mut q = QueueState::new();
    q.ingest(&files);
    let state = Mutex::new(q);

    let mut events = 0;
    run_queue(&state, |_| Ok("t".into()), |_jobs| events += 1);
    assert!(
        events >= 4,
        "at least start+finish per job, got {events}"
    );
}

#[test]
fn clear_finished_keeps_pending_and_active() {
    let dir = tempfile::tempdir().unwrap();
    let files = make_files(dir.path(), &["a.mp3", "b.mp3", "c.mp3"]);

    let mut q = QueueState::new();
    q.ingest(&files);

    let job = q.start_next().unwrap();
    q.finish(job.id, JobStatus::Failed("x".into()), String::new());

    q.clear_finished();
    assert_eq!(q.jobs().len(), 2);
    assert!(q.jobs().iter().all(|j| j.status == JobStatus::Queued));
}

#[test]
fn cancel_queued_drops_only_queued_jobs() {
    let dir = tempfile::tempdir().unwrap();
    let files = make_files(dir.path(), &["a.mp3", "b.mp3"]);

    let mut q = QueueState::new();
    q.ingest(&files);
    let active = q.start_next().unwrap();

    let removed = q.cancel_queued();
    assert_eq!(removed, 1);
    assert_eq!(q.jobs().len(), 1);
    assert_eq!(q.jobs()[0].id, active.id);
    assert!(q.has_active_work());
}
