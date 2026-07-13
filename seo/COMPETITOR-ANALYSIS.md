# StenoDrop — Competitor Analysis

Date: 2026-07-14 · Method: SERP research (no paid keyword API connected)

## The market splits into three lanes

1. **File transcription (our lane):** MacWhisper, Aiko, noScribe
2. **Live dictation (adjacent, not us):** superwhisper, Wispr Flow, Voicy, Voibe, the four "Steno" apps
3. **Cloud/web upload (what we replace):** TurboScribe, HappyScribe, Transkriptor, Rev

## Direct competitors

### MacWhisper — category leader
- €59 one-time (free tier limited). Drag-in file transcription, batch, watch
  folders, speaker diarization (Pyannote), SRT export, AI summaries.
- Owns most "whisper mac" SERPs; dozens of "MacWhisper alternative" listicles
  exist, which is itself our opportunity (those pages take submissions).
- **Our wedge:** free, simpler, folder-recursive with .txt-beside-file output.
  We will not out-feature it; we out-simple and out-free it.

### Aiko (Sindre Sorhus) — the free incumbent
- Free, on-device, no limits, App Store distribution, huge developer trust.
- **Our wedge:** batch folders (Aiko is one-at-a-time oriented), translate
  toggle UX, Urdu/South-Asian language positioning, .txt saved next to source.
  Aiko does not do folder-drop batch runs with per-file outputs.

### noScribe — academic niche
- Free, open source, interview-focused with speaker ID; clunkier UX (Python).
- **Our wedge:** native Mac feel, zero-setup beyond brew, non-academic users.

## Indirect but SERP-relevant

- **Cloud tools (TurboScribe et al.):** rank for "transcribe audio to text
  free" head terms with big content operations. Do not fight them head-on;
  their weakness is the privacy/upload/limits angle and "free" that isn't.
- **Dictation apps:** irrelevant functionally but pollute "whisper mac app"
  SERPs; our copy must always say *files/folders*, not *dictation*, to match
  intent correctly.

## E-E-A-T / distribution signals competitors have that we lack

| Signal | MacWhisper | Aiko | StenoDrop today |
|---|---|---|---|
| Custom domain | yes | App Store page | github.io subpath |
| Notarized/signed build | yes | yes | ad-hoc only |
| Homebrew cask | yes | yes | no |
| awesome-whisper listing | yes | yes | no |
| Review/listicle presence | heavy | heavy | none |
| GitHub stars as social proof | n/a | high | ~0 |

Closing these table gaps is worth more than any blog post in months 1–2.

## Keyword gaps competitors leave open (qualitative)

- **"urdu audio to english text" / "transcribe urdu voice notes"** — no Mac
  app targets this; SERPs are low-quality web converters. Same for Punjabi
  and Pashto. This is StenoDrop's most defensible organic wedge.
- **"batch transcribe audio files mac" / "transcribe a folder of audio"** —
  MacWhisper mentions batch as a feature bullet; nobody owns a dedicated page.
- **"transcribe whatsapp voice notes mac"** — high-intent, diaspora-heavy
  query; .opus support is already in the app. Nobody owns it natively.
- **"macwhisper free alternative"** — evergreen comparison intent; listicles
  rank, no first-party free product page does.
