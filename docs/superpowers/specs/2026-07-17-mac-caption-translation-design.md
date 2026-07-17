# StenoDrop Mac — Translate Downloaded Caption Files (.srt/.vtt)

Date: 2026-07-17 · Status: FROZEN for implementation after two adversarial
review rounds (r1: 33 findings/11 revisions; r2: 31 findings/11 revisions,
several verified against a live yt-dlp download). Do not deviate from
normative rules below without surfacing the deviation.

## Why this exists

Users can already download a YouTube video's captions (via yt-dlp, browser
extensions, or YouTube Studio for their own videos) as a `.srt` or `.vtt`
file. There's no audio to run through whisper — the file already **is** a
transcript with per-line timing. The ask: drop that file into StenoDrop
and get back cleaned and translated caption files, still correctly timed,
usable for re-upload as subtitles.

Two hard facts shape everything below:

1. YouTube **auto-generated** captions download as "rolling" captions with
   a specific, structurally marked duplication format (next section).
2. Apple's Translation framework **does not support Urdu, Bengali,
   Persian, Punjabi, or Pashto** in any direction. The flagship
   "Urdu captions → English" scenario cannot be served by translation in
   v1. The feature stays valuable for those languages because the
   cleaned, deduplicated source-language track is itself the primary
   output (see Output).

## The real input format (ground truth, verified against real files)

A yt-dlp-downloaded auto-caption `.vtt` is a strict alternation of:

- a **building cue** (a few seconds long) whose payload is TWO lines:
  line 1 is the *previous* completed line repeated verbatim as plain
  text; line 2 is the *new* line, usually carrying inline
  timestamp/`<c>` tags, e.g.
  `we<00:00:00.960><c> shall</c><00:00:01.500><c> fight</c>`;
- a **static cue** (~10 ms) holding just the completed plain line,
  sometimes with a filler line that is `&nbsp;` or a single space.

Verified caveats (from a live download, checked in as a fixture):
- yt-dlp only tags tokens *after* a line's first word, so a
  single-token new line (`Yeah`, `thinking`, `[Music]`) carries **zero**
  inline tags. Tag presence is a keep signal, never the only one.
- A cue's only payload can be an untagged `[Music]` line lasting 18+ s.
- Duplication is whole-line and verbatim; tags can split mid-word
  (`TH<c>E </c><c>SE</c><c>RG</c>…` for `THE SERGEANT`).
- Manually-authored `.vtt` files may be tag-free, or may legitimately
  use inline timestamp tags with no rolling duplication (karaoke /
  word-highlight files). Neither may be altered by reflow.
- SRT files from `yt-dlp --convert-subs srt` have tags stripped but
  **keep** the whole-line duplication.

## Architecture

### 1. Parsing (`CaptionFile`, new — pure, no I/O in the core)

```swift
struct CueLine {
    let text: String          // tag-free text (see tag stripping)
    let hadInlineTimestamps: Bool
}
struct Cue {
    let startMs: Int          // integer milliseconds — never Double
    let endMs: Int
    let lines: [CueLine]
}
```

- **Character encoding (decode ladder, normative)**: (1) BOM-sniff
  UTF-8 / UTF-16LE / UTF-16BE and decode accordingly; (2) otherwise
  attempt strict UTF-8; (3) fall back to `windowsCP1252` (never fails,
  covers Latin-1) and attach a job warning ("decoded as Windows-1252").
  Output is always UTF-8, no BOM, `\n` line endings. Fixtures:
  UTF-16LE-BOM SRT; Windows-1252 SRT with accented characters.
- **Timestamps**: SRT `HH:MM:SS,mmm` (comma); VTT `(HH:)?MM:SS.mmm`
  (dot, hours OPTIONAL, tolerate >2-digit hours). Stored as integer ms;
  rounding, never truncation. Timestamp string → ms → string is an
  identity round-trip (tested, including expected end-times pinned for
  the canonical real fixture).
- **Byte/line level**: strip a leading U+FEFF post-decode; normalize
  `\r\n` and bare `\r` to `\n` before block splitting; EOF is an
  implicit block terminator.
- **VTT grammar**: the `WEBVTT` header is a *block* — yt-dlp emits
  `WEBVTT\nKind: captions\nLanguage: en` — consume to the first blank
  line, tolerate `WEBVTT <text>`. Capture the `Language:` value (feeds
  source-language priority). Skip `NOTE`/`STYLE`/`REGION` blocks. Cue
  identifiers and cue settings (`align:` etc.) parsed past and dropped.
- **Inline tag stripping**: pure deletion of tag spans with
  byte-for-byte concatenation — no separator insertion, no trimming
  inside the line (tags split mid-word). Fixture uses the real
  character-chunked sample.
- **Entities**: decode `&amp;` `&lt;` `&gt;` `&nbsp;` `&lrm;` `&rlm;`
  and numeric forms on parse; re-escape `&` and `<` on VTT
  serialization (SRT needs none).
- **Emptiness predicate**: empty after entity decoding and trimming
  Unicode whitespace *including* U+00A0 and zero-width (Cf) characters.
  Explicit in tests.
- **Dropped on output (enumerated)**: styling tags, cue settings,
  cue identifiers, word-level timestamps, ruby. `<v Speaker>` is
  content: preserved as a `Speaker: ` prefix. Output is
  structure-preserving plain text in the same container format.
- **SRT indices**: ignored on parse; regenerated `1..N` on write.

### 2. Rolling-caption reflow (pure `[Cue] -> [Cue]`, deterministic)

**Dispatch is structural, never format-named.** The same rolling-run
detector gates both containers; all reflow rules apply **only inside
detected runs**. Everything outside runs — including the entirety of
tag-free manual files and karaoke files — passes through byte-identical
modulo §1's enumerated stripping.

**Named constant ε = 1000 ms** governs every timing judgment below
(run continuity, extension, folding), with explicit tolerance for 1 ms
rounding seams from comma↔dot conversion.

**Run detector (normative)**: a rolling run is ≥3 consecutive
*line-shift pairs*. A line-shift pair: block N+1 has ≥2 lines, its
FIRST line whole-line-equals block N's last emitted line, AND it
contributes at least one new line. Bare equality between single-line
blocks (chants: four contiguous `Hey!` cues) never counts — the chant
fixture must pass through byte-identical. On VTT, a run is the maximal
sequence of building cues with inter-cue gap ≤ ε *after* static-cue
removal; the alternating ~10 ms static echo cues are part of the
signature. A time-overlapping cue pair (simultaneous speakers — legal
VTT) is *transparent* for run-membership counting: the run continues
through it; only that pair's dedup/fold is skipped.

**Keep/drop inside a run**: a line is kept if it has inline timestamps
OR it is non-blank and not whole-line-equal to the previously emitted
line (the verbatim-dedup test — this is what preserves untagged
one-token lines and `[Music]`). Only exact duplicates and empty lines
are dropped. Cues left with no lines are dropped; their range folds
into the **previous** cue only when contiguous (≤ ε), else discarded.
The dedup equality test itself is **gap-independent** (a run-initial
block's first line is compared against the last globally emitted line);
ε governs timing rewrites only.

**SRT path** (tags stripped, duplication retained): the same detector
and the verbatim-dedup rule — drop a block's first line when it equals
the previous block's last emitted line; drop blocks contributing no new
line, folding as above.

**Timing**: within a run, a completed line spans from its building
cue's start to the next building cue's start when the gap ≤ ε;
otherwise the cue keeps its own end. Inter-run gaps (silence, music,
scene breaks) are **preserved**. Timing outside runs is untouched.
After any rewrite, `endMs` clamped ≥ `startMs`. Cues sorted by start
before processing.

**Normative pass-through guarantees (each is a fixture)**: tag-free
manual VTT → byte-identical; karaoke VTT → preserved; chant SRT →
byte-identical; untagged `[Music]` cue and one-token untagged new line
inside a run → preserved; silence gap → preserved.

### 3. Source language — explicit, prioritized, plumbed end-to-end

Priority: (1) VTT `Language:` header; (2) the toolbar Language picker
when not "auto" (the override UI already exists — auto-detect misfires
on exactly this user's content, Urdu heard as Hindi, and romanized Urdu
fools NLLanguageRecognizer too); (3) `NLLanguageRecognizer` on the
reflowed text, run inside `Task.detached` with the recognizer created
locally (not Sendable; `JobQueue` is `@MainActor`).

Language comparison is by `Locale.Language` components, never raw
strings (`zh-Hans` vs picker `zh`). Detection nil / out-of-list:
proceed, pass what we have, let the session decide. The detected/used
source is displayed in the job row.

### 4. Availability gate + honest scoping (the Urdu problem)

Apple Translation's supported set is ~25 languages. **ur, bn, fa, pa,
ps — all in `JobQueue.languages` — are unsupported as source or
target.**

- Pre-flight per target: `LanguageAvailability.status(from:to:)`; fail
  that language fast with a per-language user-facing message ("Apple
  Translation doesn't support Urdu"), surfaced via job warnings, before
  any translate call.
- **Nil-source branch (normative)**: `status(from:to:)` takes a
  non-optional source. When §3 resolution returns nil, call
  `status(for: <sample of reflowed text>, to: target)` (macOS 15,
  infers from sample) per target; on throw or indeterminate, fall
  through to session-side failure handled by the per-language warning
  path.
- The cleaned source track is always produced, so unsupported-language
  files still get the dedup value.
- v1 accepts the limitation; a non-Apple engine is a recorded
  follow-up. Related pre-existing exposure in the audio translate menu
  (ur/bn/fa/pa/ps targets) is filed as its own issue, not fixed here.

### 5. Translation unit — chunking and redistribution (normative)

`translations(from:)` translates each Request independently; reflowed
rolling cues are 3-8 word punctuation-free fragments, so per-cue
translation yields word salad and SOV/SVO reordering breaks cue-text ↔
timing correspondence. Therefore chunks, then redistribute:

**Chunking**:
- Boundary set, exact: Unicode `Sentence_Terminal=Yes` (explicitly
  covering `. ! ? … 。 ！ ？ ۔ ؟ ।`) — AND any inter-cue gap ≥ ε (same
  constant as §2) is an unconditional boundary regardless of
  punctuation (auto-captions are largely punctuation-free; without the
  gap rule a lecture degenerates into one giant chunk).
- Hard cap: 600 characters or 20 cues, whichever hits first, with a
  deterministic forced split at the nearest word boundary.
- Chunk boundaries are a strict superset of reflow run boundaries — a
  chunk never spans two rolling runs.
- Cues that time-overlap a neighbor are never merged into a multi-cue
  chunk (each is its own single-cue chunk).
- The `Speaker: ` prefix from `<v>` is stripped before building chunk
  text, excluded from weights, re-attached verbatim to that cue's
  translated text.
- Chunk text = cue line texts joined with single spaces; one chunk =
  one translator string, no internal newlines. Redistributed cue text
  is single-line; no wrapping in v1.

**Redistribution (an algorithm, not an adjective)**:
- Weights: per-cue cleaned source text length in **grapheme clusters**
  (Swift `Character` count), excluding Cf/format characters.
- Cumulative proportional offsets into the translated string; snap each
  offset to the nearest word boundary via locale-aware segmentation
  (`NLTokenizer`/`enumerateSubstrings(.byWords)` with the *target*
  language — handles spaceless ja/zh/ko/th); never split inside a
  grapheme cluster; ties toward the earlier boundary; rounding
  remainders by largest-remainder.
- Empty-cue rule: a cue that would receive empty/whitespace-only text
  (including zero-weight source cues) is dropped from that language's
  output; the previous surviving cue in the chunk extends `endMs` to
  cover it (fold-forward, mirroring §2), clamped ≥ `startMs`. A chunk
  whose entire translation is empty emits the cleaned source text for
  those cues with a `doneWithWarning` note.
- Redistribution assigns **text only**: every surviving output cue's
  `startMs`/`endMs` equals the reflowed source cue's.
- Worked example (normative, in tests): source cues of grapheme lengths
  [12, 5, 23] → offsets at 12/40 and 17/40 of the target string →
  snapped to nearest target-language word boundaries → three cue texts,
  none starting or ending mid-word.

**Batching & partial failure**:
- Cap each `translations(from:)` call at **300 chunks**; sub-batches
  sequenced within a language.
- Completed sub-batches are retained on failure. A failed/missing
  chunk's cues fall back to their cleaned source text; the language
  completes as `doneWithWarning` with counts ("3 of 41 segments
  untranslated"). The protocol shape must express partial results
  (per-sub-batch calls with these semantics) — a bare
  `async throws -> [String]` cannot.
- Responses matched to chunks by `clientIdentifier` (chunk index),
  never array position (fake-engine test shuffles responses; another
  omits one response).
- Per-language progress driven by sub-batch completion so the
  `.translating` row visibly moves.
- Parsing and reflow run off the main actor inside the pump's
  `Task.detached` (multi-MB folder drops must not beachball ingest).

### 6. TranslationBridge redesign (not "gains a batch method")

Known failure to fix: `.translationTask` only re-fires when the
Configuration *value* changes; two consecutive same-target requests
never trigger the second closure — continuation leak, job wedged
forever (today's audio path survives only because whisper's
seconds-long gaps drain the queue between requests). Required:

1. Guaranteed configuration transition per head-of-queue: keep the
   stored `Configuration` and call `invalidate()` for same-target
   heads (verified present at macOS 15 in the local SDK). The stored
   Configuration lives in a MainActor-isolated published property
   mutated exclusively by the single main-actor consumer below — NOT
   the current per-render computed property
   (`TranslationBridgeView.swift:38-43`), which resets the version
   each render and reintroduces the wedge.
2. Ordering-safe publish: replace the fire-and-forget
   `Task { @MainActor }` with an `AsyncStream` consumed by one
   main-actor task.
3. Batch-typed queue: requests carry `texts: [String]`,
   `sourceLanguage: Locale.Language?`, `target: Locale.Language`.
   `Configuration` and `Request` are NOT Sendable — only Sendable
   descriptors (`[String]`, `Locale.Language`, `UUID`) cross the
   locked-bridge boundary.
4. `Configuration(source:target:)` carries §3's explicit source
   (today's hardcoded `source: nil` goes away).
5. Manual checklist gains: two consecutive same-target requests;
   first-ever translation to a language triggers an in-session pack
   download (first batch slow; pack-missing fails per-language, never
   the whole job).

### 7. Job model — unified, with the full call-site list

One heterogeneous collection: `enum Job { case audio(TranscriptionJob);
case captions(CaptionJob) }` (or shared protocol), processed by the
existing single-flight `pump()` (which also serializes bridge access).
`JobStatus` gains `.translating` with per-language progress.

**Enumerated call sites (all in scope)**:
- `jobs` array sites: quit guard `hasActiveWork`
  (JobQueue.swift:79-81 → StenoDropApp.swift), ingest dedupe
  (`pendingPaths`), drop-zone empty state, `clearFinished`/
  `hasFinishedJobs`, the `ForEach`.
- `JobStatus.isActive` (TranscriptionJob.swift:22-24) is an `==` chain
  that compiles unchanged when `.translating` is added and then
  **silently breaks the quit guard** (app quits mid-batch without
  warning). It becomes an exhaustive `switch` (pattern-match, not `==`,
  since `.translating` carries progress values — Equatable comparisons
  change meaning). Unit test: `JobStatus.translating(...).isActive ==
  true`.
- `== .queued` sites (JobQueue.swift:80, 160).
- `pump()`'s in-place mutations (JobQueue.swift:167, 192, 194, 233) —
  an enum forces extract-mutate-reassign or a settable-status protocol
  and a per-kind pump branch.
- `outputURL(for:)` (JobQueue.swift:120-126) — see §8.
- `JobRowView` — typed `let job: TranscriptionJob` parameter; the
  Reveal-in-Finder gate at JobRowView.swift:52 is `status == .done`,
  but `doneWithWarning` is the COMMON caption success outcome
  (Urdu-unsupported note, English-skipped note): **Reveal must also
  show for `.doneWithWarning`**. `doneWithWarning`'s label
  ("Done (not saved)") is wrong for caption jobs — make it
  job-kind-aware. "Saved <file>"/"Reveal .txt" strings generalize.
- Caption row: expanded view shows reflowed cue text; detected/used
  source language shown; per-language failures via `doneWithWarning`.

### 8. Ingest, output naming, re-run semantics

- **Both filter sites**: the single-file branch in `ingest(urls:)`
  (JobQueue.swift:94) AND the directory enumerator `audioFiles(in:)`
  (JobQueue.swift:137-150). Copy sweep: the drop notice, drop-zone
  copy, NSOpenPanel prompt/message, translate-menu help text.
- **Language-code stripping helper (shared, tested)**: strip one
  recognized trailing ISO code from a basename (`Talk.en` → `Talk`).
  Used by output naming AND the mixed-folder rule.
- **Mixed folders**: compare *stripped* basenames (yt-dlp always writes
  `<name>.<lang>.<ext>`, so unstripped comparison matches zero real
  folders). `Talk.mp4` + `Talk.en.vtt` in one drop → only the caption
  file queues, noted in the drop notice; `Talk.part2.vtt` does not
  match `Talk.mp4`.
- **Two-phase output claiming**: target-language paths (known at
  enqueue from `targetLanguages`) are claimed at enqueue; the
  source-track path is claimed at source-resolution time under
  identical collision rules, surfaced in the job row. Every job (audio
  included) exposes its full prospective output set; the collision
  check unions over those sets (fixes the pre-existing audio hole where
  per-language outputs are derived at write time and never claimed).
- **Re-run semantics (decided)**: deterministic caption output names
  (stripped basename + target code + container ext, plus the `.txt`
  adjunct) are **app-owned and overwritten on re-run** — exactly how
  the audio pipeline already overwrites `song.txt`. The
  no-silent-overwrite rule applies to collisions with OTHER queued
  jobs' sources/outputs and to pre-existing files whose names do NOT
  match this job's own deterministic output set. Cross-launch re-runs
  therefore overwrite same-named caption outputs. Fallback for genuine
  collisions: the existing `appendingPathExtension` disambiguation.

### 9. Output

- **Always** emit the reflowed source-language file
  (`name.<sourceLang>.srt`/`.vtt`) — the cleaned track is the primary
  artifact and, for unsupported-translation languages, the whole
  feature.
- One timed file per successfully translated target, same container
  format as input.
- Same-as-source target skipped **with a visible note** ("English
  skipped — source is already English; cleaned track saved").
- Cheap adjunct: flattened plain-text `name.<sourceLang>.txt`.
- The original downloaded file is never modified.

## Malformed-input policy

Per-cue best-effort: skip unparseable blocks, count them, surface via
`doneWithWarning`. Whole-file reject (job failed, `showNotice` pattern)
only when zero valid cues parse — never write N empty output files.

## Testing plan (TDD; council-hardened, two rounds)

Unit-tested and pure: decode ladder (UTF-16LE BOM, Windows-1252);
timestamp round-trip identity with pinned end-times for the real
fixture; BOM/CRLF/EOF; VTT header block + `Language:` capture;
NOTE/STYLE/REGION; entity decode/re-escape; mid-word tag-chunk
stripping (real sample); emptiness predicate (U+00A0, zero-width); run
detector (line-shift pair definition, chant false-positive, overlap
transparency, VTT static-cue signature); keep/drop rules (untagged
one-token line, `[Music]`, karaoke pass-through, tag-free pass-through,
byte-identical guarantees); timing (ε extension, gap preservation,
fold direction + contiguity, clamping); chunking (boundary set,
gap rule, hard cap, run-boundary superset, overlap-solo, Speaker
prefix); redistribution (worked example, grapheme weights, boundary
snapping incl. ja target, empty-cue fold-forward, no mid-word
starts/ends, text-only invariant); batching (clientIdentifier shuffle,
missing response, sub-batch partial retention); naming (lang-code
stripping, two-phase claiming, re-run overwrite, mixed-folder);
`isActive` exhaustive-switch test; ingest routing both sites.

**Real fixtures (mandated)**: the live yt-dlp download verified during
review (`fixture2.en.vtt` — copy from session scratchpad
`/private/tmp/claude-501/-Users-isupercoder/d3d67335-0e61-464a-b9da-1d6f99a56c3e/scratchpad/fixture2.en.vtt`
into the test target as the FIRST implementation commit, before
scratchpad garbage collection) plus the captured C-SPAN roll-up sample
(`real-rollup-sample.vtt`, same scratchpad `fixtures/` dir). Follow-ups
when downloadable: an Urdu track, a Japanese video, a music video with
a long instrumental gap, and a sparse-burst conversational file
(vlog/Q&A pacing — where the ≥3-pair threshold is most likely to
under-fire; if it does, allow detection evidence to span >ε gaps while
keeping ε strictly timing-local).

NOT covered by automated tests (same boundary as the audio feature):
real `NLLanguageRecognizer` accuracy, real `TranslationSession` output
and pack-download behavior — manual on-hardware checklist, including
two consecutive same-target requests.

## Build sequencing (for the implementation fleet)

Land shared types FIRST (Cue/CueLine, the revised TranslationEngine
protocol with batch/partial-result shape, the Job enum skeleton) before
parallelizing — `TranslationEngine.swift` is the only cross-workstream
file. Verified: no other `jobs`-array consumers beyond the enumerated
sites (RecordingController only reads `translatesToEnglish`/
`languageCode`). Clean seams: parser / reflow / chunking+redistribution
/ bridge / integration.

## Ship-time checklist (outside Sources/)

`mac/README.md` (inputs are no longer audio/video only);
`scripts/` gains a caption smoke test against the checked-in fixture;
`make-app.sh` CFBundleShortVersionString bump; no
`CFBundleDocumentTypes` declared → Finder "Open With"/dock drop of
`.srt`/`.vtt` won't work — recorded as a known limitation (or fixed);
site/release notes.

## Out of scope (v1)

- Translation engine for Apple-unsupported languages (ur, bn, fa, pa,
  ps) — recorded follow-up; v1 ships the gate + cleaned track.
- Formats other than `.srt`/`.vtt`.
- Fetching captions from YouTube URLs — the app never contacts YouTube.
- Styling-faithful output; line wrapping of translated cues.
- Cancel button (pre-existing app-wide gap, backlog).
- Picker-level "unsupported language" affordances (folded into the
  separately filed audio-picker issue).
