// StenoDrop desktop UI — plain JS over the global Tauri API.
"use strict";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const dialog = window.__TAURI__.dialog;
const appWindow = window.__TAURI__.window.getCurrentWindow();

// Same 19 languages as the Mac app (JobQueue.swift), plus auto-detect.
const LANGUAGES = [
  ["auto", "Auto-detect"],
  ["ur", "Urdu"], ["en", "English"], ["ar", "Arabic"], ["bn", "Bengali"],
  ["zh", "Chinese"], ["fr", "French"], ["de", "German"], ["hi", "Hindi"],
  ["id", "Indonesian"], ["it", "Italian"], ["ja", "Japanese"],
  ["ko", "Korean"], ["fa", "Persian"], ["pt", "Portuguese"],
  ["pa", "Punjabi"], ["ps", "Pashto"], ["ru", "Russian"],
  ["es", "Spanish"], ["tr", "Turkish"],
];

const AUDIO_EXTENSIONS = [
  "wav", "mp3", "m4a", "m4b", "aac", "flac", "ogg", "oga", "opus", "aiff",
  "aif", "caf", "amr", "wma", "3gp", "mp4", "mov", "m4v", "avi", "webm", "mkv",
];

const $ = (id) => document.getElementById(id);

// ---------- settings (persisted in localStorage) ----------

function loadSettings() {
  const translate = localStorage.getItem("translateToEnglish");
  const saved = localStorage.getItem("languageCode") || "auto";
  return {
    translate: translate === null ? true : translate === "true",
    language: LANGUAGES.some(([c]) => c === saved) ? saved : "auto",
  };
}

async function pushSettings() {
  const settings = {
    translate: $("translate-toggle").checked,
    language: $("language-select").value,
  };
  localStorage.setItem("translateToEnglish", String(settings.translate));
  localStorage.setItem("languageCode", settings.language);
  await invoke("set_settings", settings);
}

// ---------- rendering ----------

const STAMP = {
  queued: { label: "Queued", cls: "" },
  transcribing: { label: "Transcribing", cls: "stamp-transcribing" },
  done: { label: "Done", cls: "stamp-done" },
  done_with_warning: { label: "Not saved", cls: "stamp-warning" },
  failed: { label: "Failed", cls: "stamp-failed" },
};

function baseName(path) {
  return path.split(/[\\/]/).pop();
}

function renderJobs(jobs) {
  const list = $("job-list");
  list.textContent = "";
  for (const job of jobs) {
    const li = document.createElement("li");
    li.className = "job-row";

    const name = document.createElement("span");
    name.className = "job-name";
    name.textContent = baseName(job.source);
    name.title = job.source;

    const detail = document.createElement("span");
    detail.className = "job-detail";
    if (job.status.kind === "failed" || job.status.kind === "done_with_warning") {
      detail.textContent = job.status.detail;
      detail.title = job.status.detail;
    } else if (job.status.kind === "done") {
      detail.textContent = baseName(job.output);
      detail.title = job.output;
    }

    const stamp = document.createElement("span");
    const meta = STAMP[job.status.kind] || STAMP.queued;
    stamp.className = `stamp ${meta.cls}`;
    stamp.textContent = meta.label;

    li.append(name, detail, stamp);
    list.appendChild(li);
  }

  $("empty-docket").classList.toggle("hidden", jobs.length > 0);
  $("docket-count").textContent =
    `DOCKET — ${jobs.length} FILE${jobs.length === 1 ? "" : "S"}`;
  $("clear-btn").classList.toggle(
    "hidden",
    !jobs.some((j) => ["done", "done_with_warning", "failed"].includes(j.status.kind)),
  );
  $("cancel-btn").classList.toggle(
    "hidden",
    !jobs.some((j) => j.status.kind === "queued"),
  );
}

let noticeTimer = null;
function showNotice(text) {
  const el = $("notice");
  el.textContent = text;
  el.classList.remove("hidden");
  clearTimeout(noticeTimer);
  noticeTimer = setTimeout(() => el.classList.add("hidden"), 5000);
}

// ---------- ingest ----------

async function ingestPaths(paths) {
  if (!paths || paths.length === 0) return;
  const added = await invoke("ingest_paths", { paths });
  if (added === 0) showNotice("No supported audio files in that drop.");
}

async function pickFiles() {
  const picked = await dialog.open({
    multiple: true,
    filters: [{ name: "Audio & video", extensions: AUDIO_EXTENSIONS }],
  });
  if (picked) await ingestPaths(Array.isArray(picked) ? picked : [picked]);
}

async function pickFolder() {
  const picked = await dialog.open({ directory: true, multiple: true });
  if (picked) await ingestPaths(Array.isArray(picked) ? picked : [picked]);
}

// ---------- setup screen ----------

function showScreen(ready) {
  $("setup-screen").classList.toggle("hidden", ready);
  $("main-screen").classList.toggle("hidden", !ready);
}

async function startDownload() {
  $("download-btn").disabled = true;
  $("download-error").classList.add("hidden");
  $("download-progress").classList.remove("hidden");
  await invoke("download_model");
}

function onModelProgress(p) {
  if (p.error) {
    $("download-error").textContent = p.error;
    $("download-error").classList.remove("hidden");
    $("download-progress").classList.add("hidden");
    $("download-btn").disabled = false;
    $("download-btn").textContent = "Try again";
    return;
  }
  if (p.done) {
    showScreen(true);
    return;
  }
  const pct = Math.floor((p.fraction || 0) * 100);
  $("progress-fill").style.width = `${pct}%`;
  $("progress-label").textContent =
    p.total ? `${pct}% — ${Math.round(p.downloaded / 1e6)} of ${Math.round(p.total / 1e6)} MB` : `${pct}%`;
}

// ---------- boot ----------

async function main() {
  // Language picker
  const select = $("language-select");
  for (const [code, name] of LANGUAGES) {
    const opt = document.createElement("option");
    opt.value = code;
    opt.textContent = name;
    select.appendChild(opt);
  }
  const settings = loadSettings();
  select.value = settings.language;
  $("translate-toggle").checked = settings.translate;
  await pushSettings();

  select.addEventListener("change", pushSettings);
  $("translate-toggle").addEventListener("change", pushSettings);

  // Drop zone + pickers
  const zone = $("drop-zone");
  zone.addEventListener("click", pickFiles);
  zone.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") pickFiles();
  });
  $("add-files-btn").addEventListener("click", (e) => { e.stopPropagation(); pickFiles(); });
  $("add-folder-btn").addEventListener("click", (e) => { e.stopPropagation(); pickFolder(); });

  await listen("tauri://drag-enter", () => zone.classList.add("armed"));
  await listen("tauri://drag-leave", () => zone.classList.remove("armed"));
  await listen("tauri://drag-drop", (e) => {
    zone.classList.remove("armed");
    ingestPaths(e.payload.paths);
  });

  // Queue events
  await listen("queue-changed", (e) => renderJobs(e.payload));
  $("clear-btn").addEventListener("click", () => invoke("clear_finished"));
  $("cancel-btn").addEventListener("click", () => invoke("cancel_queued"));

  // Model / setup
  await listen("model-progress", (e) => onModelProgress(e.payload));
  $("download-btn").addEventListener("click", startDownload);
  showScreen(await invoke("model_ready"));

  // Quit-mid-batch confirmation
  await appWindow.onCloseRequested(async (event) => {
    // preventDefault must run before the first await; we re-close manually.
    event.preventDefault();
    const busy = await invoke("has_active_work");
    if (!busy) {
      await appWindow.destroy();
      return;
    }
    const quit = await dialog.ask(
      "Transcription is still running. Quit anyway?",
      { title: "StenoDrop", kind: "warning" },
    );
    if (quit) await appWindow.destroy();
  });

  renderJobs(await invoke("get_jobs"));
}

main();
