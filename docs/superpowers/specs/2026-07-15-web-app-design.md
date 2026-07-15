# StenoDrop Web — Design Spec

Date: 2026-07-15 · Status: Approved (user request: web app, hosted free)

## Goal

Same promise as every other platform — drop audio files or a folder, get
transcripts back, nothing leaves the device — but running entirely in the
browser, no install, no server, no ongoing cost.

## Architecture decision

- **Client-side only.** No backend. Transcription runs in the visitor's
  own browser via WebAssembly/WebGPU. This is the only option consistent
  with the "free forever, nothing uploaded" brand promise — a server-side
  version would mean per-user compute cost and a real privacy regression
  (audio would leave the device).
- **Engine: Transformers.js** (`@huggingface/transformers`, ESM via CDN,
  no build step — matches the rest of this project's "plain HTML/CSS/JS"
  philosophy). Runs Whisper small via ONNX Runtime Web, WebGPU with WASM
  fallback. This is a deliberate divergence from whisper.cpp (used by
  Mac/Windows/Linux/Android) — porting whisper.cpp itself to WASM is a
  much larger, more fragile undertaking than using an actively-maintained
  browser-native runtime. Same model family, same 100 languages, same
  translate capability; different file format (ONNX, not GGML) and a
  different (likely somewhat smaller, browser-appropriate) download size.
- **Hosted at `fahdi.github.io/stenodrop/app/`** — lives in this repo at
  `docs/app/`, deployed by the same GitHub Pages pipeline as the
  marketing site. No new repo, no new hosting account.
- **Web Worker** runs the model load + inference so the UI thread stays
  responsive during a transcription.
- **Model caching**: Transformers.js caches the downloaded model via the
  browser Cache API automatically — no custom caching code needed, but
  the UI should make clear the first run downloads it and later visits
  are instant.

## Constraints unique to the browser (vs. native apps)

- **No writing beside the source file.** Browsers can't do that. Output
  is a downloadable `.txt` per file, plus a "Download all as .zip" button
  once a batch completes (small ESM-importable zip library via CDN is an
  acceptable exception to "no dependencies" — writing a ZIP by hand is
  not worth the risk).
- **Folder picking** via `<input webkitdirectory>` (works cross-browser
  for read access; the File System Access API's full read/write directory
  handle is Chromium-only, so don't depend on it for the core flow).
- **No true background operation** — closing the tab stops an in-flight
  batch. Acceptable for a "try it now, no install" on-ramp; the native
  apps remain the answer for long unattended batches.

## UI

Same brand system as the marketing page (reuse the CSS custom properties,
Besley + Courier Prime, court-transcript identity) — either embedded
directly in `docs/app/index.html` or sharing a small CSS file with the
main site. Same shape as the other apps: drop zone / folder picker,
language picker (same 19-language list + auto) + translate toggle
defaulting on, batch queue with per-file status, in-page transcript
view + copy, zip-all-download once finished.

## Marketing site integration

Add a clear "Try it in your browser — no install" path from the main
site (hero-level mention and/or a 5th card in the download grid). Must
not break the pixel-aligned 4-card download grid or introduce any new
horizontal-overflow/mobile regressions — re-verify at all previously
tested breakpoints (320–1440px) after the change.

## Verification

Must actually transcribe a real short audio clip end-to-end in a real
browser (Playwright) before this is considered done — not just "the page
loads." A synthesized short speech clip (e.g. via `say` on macOS,
converted to a browser-friendly format) run through the full pipeline
should produce recognizable text.

## Out of scope (v1)

Live recording, WhatsApp-style sharing, PWA/offline-install support,
IndexedDB fallback beyond what Transformers.js already provides,
non-English UI localization.
