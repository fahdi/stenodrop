# StenoDrop

Free, offline transcription for people with folders full of recordings.
Drop in audio files, whole directories, or downloaded YouTube captions;
StenoDrop transcribes and translates everything on your own machine.
Nothing is uploaded, nothing is metered, and the transcript lands right
next to each source file.

**[Website](https://fahdi.github.io/stenodrop/)** ·
**[Try it in your browser](https://fahdi.github.io/stenodrop/app/)** ·
**[Downloads](https://github.com/fahdi/stenodrop/releases/latest)**

## What it does

- **Batch transcription.** Point it at a year of voice memos. Every audio
  file (and the audio track of your videos) becomes a `.txt` transcript
  saved beside the original. About 100 languages, with first-class Urdu.
- **Translation, on device.** One toggle turns any spoken language into
  clean English text. On the Mac app you can also check multiple target
  languages and get one transcript file per language, translated locally.
- **YouTube caption cleanup.** Auto-generated captions download with
  every line doubled. Drop the `.srt` or `.vtt` on StenoDrop and get a
  clean, correctly timed track, plus translated versions ready to
  re-upload. Manually authored files pass through untouched.
- **Models by capability.** Efficient for quick single-language work,
  Enhanced for accents and noisy rooms, Maximum for multilingual and
  Indic-language audio with code-switching. You choose; only the small
  one downloads by default.
- **Live recording.** The Mac app records from the microphone with a
  rolling transcript, then saves the audio and text together.
- **Private by construction.** Transcription and translation run
  entirely on your machine. Airplane mode works fine.

## Get it

| Platform | Status | Where |
|---|---|---|
| macOS 15+ | Native Swift app | [Releases](https://github.com/fahdi/stenodrop/releases/latest) |
| Windows 10/11 | Tauri app | [Releases](https://github.com/fahdi/stenodrop/releases/latest) |
| Linux | AppImage / .deb | [Releases](https://github.com/fahdi/stenodrop/releases/latest) |
| Android 8+ | Beta APK, sideload | [stenodrop-android](https://github.com/fahdi/stenodrop-android/releases) |
| Any browser | No install | [Web app](https://fahdi.github.io/stenodrop/app/) |
| iPhone | Planned | [stenodrop-ios](https://github.com/fahdi/stenodrop-ios) |

Betas are unsigned: on macOS, right-click and Open the first time; on
Windows, click through SmartScreen.

Known limits, stated plainly: Apple's on-device translation covers about
25 target languages and does not include Urdu, Bengali, Persian,
Punjabi, or Pashto. Caption files in those languages still get the
cleaned, deduplicated track; audio in them still translates to English
through Whisper itself.

---

## For developers

This repo holds the Mac app, the Windows/Linux desktop app, the website,
and the browser app. Android lives in
[stenodrop-android](https://github.com/fahdi/stenodrop-android); the
optional cloud transcription server lives in
[stenodrop-server](https://github.com/fahdi/stenodrop-server).

```
mac/        Native macOS app (SwiftPM, no .xcodeproj), whisper-cli engine
desktop/    Windows/Linux app (Tauri v2 + Rust, whisper-rs, no ffmpeg)
docs/       Website (GitHub Pages from main:/docs) + browser app (docs/app/)
cli/        Original Python CLI, kept for reference
docs/superpowers/specs/   Design specs, one per feature, reviewed before build
```

### Mac

```bash
brew install whisper-cpp ffmpeg
cd mac
swift test                # 120+ unit tests
./scripts/e2e-test.sh     # real engine smoke test
./scripts/make-app.sh     # → dist/StenoDrop.app (ad-hoc signed)
```

Details, architecture notes, and known limitations: [mac/README.md](mac/README.md).

### Desktop (Windows/Linux)

```bash
cd desktop
npm install
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

Releases are cut by `release.yml` on `v*` tags for all three desktop
platforms.

### Web app

`docs/app/` is a static page: Whisper in the browser via Transformers.js
plus an opt-in cloud mode against stenodrop-server. Serve `docs/`
locally with any static file server.

### Contributing

Issues and pull requests are welcome. Feature work here starts from a
written spec in `docs/superpowers/specs/` and lands with tests; the
larger recent features were adversarially design-reviewed before
implementation, and the specs record what was decided and why.

MIT licensed. Built on [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
and [OpenAI Whisper](https://github.com/openai/whisper).
Built by [@fahdi](https://github.com/fahdi).
