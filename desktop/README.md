# StenoDrop Desktop (Windows + Linux)

Tauri v2 + Rust port of StenoDrop for Windows and Linux. macOS keeps the
native SwiftUI app in `../mac/` — see
`../docs/superpowers/specs/2026-07-14-cross-platform-design.md`.

Everything runs on-device: symphonia + rubato decode any supported
audio/video container to 16 kHz mono, whisper-rs (whisper.cpp, statically
linked) transcribes it, and the transcript lands as a `.txt` beside the
source file. No ffmpeg, no network use after the one-time model download.

## Layout

```
desktop/
  ui/                     plain HTML/CSS/JS frontend (no npm, no framework)
  src-tauri/
    src/
      engine/scan.rs      folder ingest: 21 extensions, hidden-skip, dedupe, sort
      engine/output.rs    txt-beside-source naming + name.ext.txt collision fallback
      engine/decode.rs    symphonia + rubato → 16 kHz mono f32
      engine/whisper.rs   whisper-rs wrapper (translate + language code)
      queue.rs            sequential queue state machine + runner
      model.rs            ggml-small download w/ progress, status+size validation
      commands.rs         tauri commands + worker thread + events
      lib.rs / main.rs    tauri entry
    tests/                integration tests + generated audio fixtures
```

## Dev setup

Prerequisites (all platforms):

- Rust stable (1.80+), `cargo`
- CMake (builds whisper.cpp for whisper-rs): `brew install cmake` /
  `winget install Kitware.CMake` / `apt install cmake build-essential`
- Tauri CLI v2: `cargo install tauri-cli --locked`

Linux additionally needs the Tauri webview stack:

```sh
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf
```

Windows additionally needs the MSVC build tools and WebView2 (preinstalled
on Windows 11).

Run in dev mode (serves `../ui` directly):

```sh
cd desktop/src-tauri
cargo tauri dev
```

## Tests

```sh
cd desktop/src-tauri
cargo test                 # unit + integration (scan/output/queue/decode/model)
```

Real-model transcription test (needs the ggml-small model in the app-data
dir — download it once via the app, or copy an existing one):

```sh
cargo test --test whisper_integration -- --ignored --nocapture
```

Model location per OS (`dirs::data_dir()`):

- macOS: `~/Library/Application Support/StenoDrop/models/ggml-small.bin`
- Windows: `%APPDATA%\StenoDrop\models\ggml-small.bin`
- Linux: `~/.local/share/StenoDrop/models/ggml-small.bin`

Audio fixtures in `src-tauri/tests/fixtures/` are committed. To regenerate:

```sh
ffmpeg -f lavfi -i "sine=frequency=440:duration=0.5:sample_rate=44100" -ac 2 -c:a pcm_s16le tone.wav
ffmpeg -f lavfi -i "sine=frequency=440:duration=0.5:sample_rate=44100" -ac 2 -c:a libmp3lame -q:a 4 tone.mp3
ffmpeg -f lavfi -i "sine=frequency=440:duration=0.5:sample_rate=44100" -ac 2 tone.flac
# jfk.wav: any ~5 s English speech clip at 16 kHz mono that contains "country"
```

## Build

```sh
cd desktop/src-tauri
cargo tauri build --no-bundle   # compile check without installers
cargo tauri build               # platform installers
```

Per-platform bundle output (`src-tauri/target/release/bundle/`):

- Windows: NSIS `.exe` installer (build on Windows)
- Linux: AppImage + `.deb` (build on Ubuntu 22.04 for widest glibc compat)
- macOS: builds and runs for development, but the shipped Mac app is `../mac/`

Cross-compilation is not supported by the webview stack — CI builds each
platform natively (see issue #13 / `release.yml`).

## Notes

- Config: `productName` StenoDrop, identifier `com.fahdi.stenodrop`,
  version 0.2.0, min window 700×520.
- Settings (language, translate toggle) persist in webview localStorage and
  are pushed to the Rust side on change.
- Symphonia has no decoder for `opus`, `amr`, `wma`, or the proprietary
  video codecs inside some containers; those files are accepted into the
  queue (same extension list as the Mac app) and fail per-file with a clear
  error while the rest of the batch continues.
