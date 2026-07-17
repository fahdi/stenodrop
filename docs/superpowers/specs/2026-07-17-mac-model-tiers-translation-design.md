# StenoDrop Mac — Model Tiers + Multi-Language Translation

Date: 2026-07-17 · Status: Approved (Mac app only; Desktop/Android parity is future work)

## Why this exists

Two asks from Fahd:

1. Let users choose a bigger/smarter model instead of being stuck on the
   one hardcoded `ggml-small`, sold by capability ("handles Indic
   languages and code-switching better") rather than by filename.
2. Let users translate one source file into several target languages at
   once, as a setting that persists across launches — not just the
   existing single "translate to English" toggle.

Scope for this pass is the **Mac app only**. `ModelDownloader.swift` /
`WhisperEngine.swift` (Mac) and `model.rs` (Desktop/Tauri) currently
mirror the same single-model design, and Android likely does too — full
cross-platform parity is real work per platform and is deliberately
deferred to follow-up issues once this lands and is proven out.

## Feature 1: Tiered model picker

### The trade-off, stated plainly (for the UI copy)

Model quality scales with size; the UI never shows filenames, only this
framing, approximate size noted but not the lead:

| Tier | Model (internal only) | ~Size | Copy |
|---|---|---|---|
| Efficient | `ggml-small` | ~500 MB | "Fast and lightweight. Great for single-language recordings." |
| Enhanced | `ggml-medium` | ~1.5 GB | "Sharper accuracy — handles accents, mixed audio, and background noise better." |
| Maximum | `ggml-large-v3-turbo` | ~1.6 GB | "Our most capable model. Built for multilingual and Indic-language audio, including code-switching." |

`large-v3-turbo` is chosen over plain `large-v3` for the top tier: it's
roughly half the size and faster, at near-identical accuracy — no
reason to make users pay the larger download for a marginal gain.

### Architecture

- `ModelDownloader` generalizes from one hardcoded `URL` to a
  `ModelTier` enum (`.efficient`, `.enhanced`, `.maximum`), each
  carrying its huggingface URL, filename, and a `minimumValidSize`
  scaled to that model (today's 400 MB floor only makes sense for
  `small`; `medium`/`large-v3-turbo` need their own floors).
- `download(tier:)` replaces `start()`; same HTTP-status + byte-size
  validation logic, parameterized per tier instead of hardcoded to one
  model.
- Selected/active tier persists via `UserDefaults` (`@AppStorage
  "modelTier"`), defaulting to whatever tier is already downloaded on
  existing installs (so nobody is silently switched to a model they
  don't have), or `.efficient` for fresh installs.
- `WhisperEngine.modelPath` / `modelIsReady` read the active tier's
  file instead of a single hardcoded path. Multiple tiers can be
  downloaded and kept on disk at once; only the active one is used for
  transcription. No auto-deletion of non-active tiers in this pass —
  users manage disk space by not downloading tiers they don't want.
- `SetupView` (first-run) keeps today's single "download the model"
  flow but downloads whatever tier is currently selected (default
  Efficient). A new Settings surface (see Feature 2 below — both
  features share one Settings view) lets users switch tiers and
  download additional ones after first run.

### Testing

`ModelDownloader`'s validation logic (HTTP status check, byte-size
floor per tier, tier→URL/filename mapping) is pure and unit-testable.
The actual `URLSession` download is exercised via dependency injection
(a protocol the tests fake) rather than hitting the network in CI.

## Feature 2: Multi-language translation, persisted

### Current state

One boolean `translateToEnglish` toggle in `JobQueue`, wired straight
into whisper.cpp's native `--translate` flag (which only ever outputs
English).

### Translation engine: Apple's on-device Translation framework

The original design considered porting NLLB-200 to run offline on Mac
— reconsidered after research (task in this spec's originating
session). Apple ships a native, fully on-device, offline **Translation
framework** (`TranslationSession`, introduced macOS 14.4, matured for
programmatic batch use in macOS 15/Sequoia) that does exactly this with
no model bundling, no download infra of our own to build, and quality
backed by Apple's own maintained language packs. This is a strictly
better fit than hand-rolling an NLLB port:

- Fully on-device once a language pack is installed — no network calls,
  no per-request cost, consistent with the rest of the app's
  privacy/offline stance.
- `LanguageAvailability` lets us check install status per language pair
  and `prepareTranslation()` lets us prompt the user to download a pack
  proactively.
- **Constraint that shapes the architecture**: `TranslationSession`
  cannot be constructed directly in arbitrary background code — it is
  created by SwiftUI's `.translationTask(_:action:)` modifier and only
  exists while a hosting view is in the hierarchy. The batch job queue
  (`JobQueue`, which runs transcription via `Task.detached` off the
  main thread) needs a bridge: a persistent, invisible view attached
  somewhere in `RootView`'s hierarchy that hosts `.translationTask`,
  fed a stream of pending translation requests and publishing results
  back to `JobQueue` via an async continuation.
- Deployment target moves from macOS 14 to **macOS 15** in
  `Package.swift` to use the mature batch API. This is a real
  compatibility trade-off worth confirming: does dropping macOS 14
  support matter given StenoDrop's current install base?
- The framework does not work in the iOS/macOS Simulator or SwiftPM
  test hosts — translation-dependent code paths are covered by unit
  tests via a `TranslationEngine` protocol that production code depends
  on and tests fake; the real `TranslationSession`-backed implementation
  is verified manually on-device, not in the automated suite.

### UI / persistence

- The single boolean toggle becomes a **multi-select language picker**
  in Settings, sourced from the same `JobQueue.languages` list already
  used for source-language selection. English is pre-selectable and
  "free" (whisper's native `--translate` flag, already implemented, no
  new engine involved). Any other selected language routes through the
  `TranslationEngine` bridge described above.
- Persists as `Set<String>` of language codes via `UserDefaults`
  (`@AppStorage` with a custom `RawRepresentable` wrapper, since
  `AppStorage` doesn't support `Set` natively), surviving app
  relaunches.
- `translateToEnglish: Bool` is removed in favor of
  `targetLanguages: Set<String>`; `en` in the set is equivalent to
  today's toggle being on.

### Pipeline

Per job: whisper produces the **original-language transcript once**
(current behavior, unchanged). Then, for each selected target language:

- `en` selected → re-run whisper with `--translate` (existing path,
  unchanged behavior/cost).
- any other language selected → original transcript text is sent
  through the `TranslationSession` bridge for that language pair.

Output: one file per language, named `filename.txt` (original,
unchanged, backward compatible with every existing job/output),
`filename.en.txt`, `filename.fr.txt`, etc. for each additional selected
target. If only the original language is wanted (no targets selected),
behavior is unchanged from today.

### Testing

TDD covers: persistence round-trip of `targetLanguages`, output
filename generation per language, and pipeline orchestration (given a
transcript and a set of target languages, the right translation calls
happen and the right files get written) against a faked
`TranslationEngine`. Actual on-device translation quality/behavior is
verified manually since it requires real hardware and a live SwiftUI
view host.

## Out of scope (this pass)

- Desktop (Tauri) and Android parity for either feature — follow-up
  issues once this is proven on Mac.
- Auto-deleting unused model tiers to reclaim disk space.
- A 4th "Ultimate" tier (`large-v3`) — `large-v3-turbo` covers the top
  end at meaningfully lower cost; revisit only if users specifically
  ask for more headroom than turbo gives.
- Translating into a language whisper didn't recognize as the source
  (translation always starts from the original-language transcript,
  not from source audio directly, to avoid double-lossy hops).
