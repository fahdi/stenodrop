# StenoDrop — Site Structure

All pages are static HTML under `docs/` (GitHub Pages root), same design
system as index.html. Flat URLs — no folders needed at this scale.

```
/                                   Home (exists)
/download.html                      Download + install + Gatekeeper note (thin now, exists on home; split when notarized)
/urdu-audio-to-english.html         T1 wedge — long-form, script samples, translate toggle demo
/batch-transcribe-folder-mac.html   T1 wedge — folder workflow, screenshots, vs one-file-at-a-time tools
/whatsapp-voice-notes-mac.html      T1 wedge — .opus how-to, export-from-phone steps
/macwhisper-free-alternative.html   T1 comparison — honest table (they win on diarization/SRT; we win on free+simple)
/transcribe-voice-memos-mac.html    T2 — Voice Memos export path + batch
/offline-transcription-mac.html     T2 — privacy angle, airplane-mode framing
/languages/                         T2 cluster (month 4+): hindi.html, punjabi.html, pashto.html, arabic.html
/faq.html                           FAQPage schema; feeds GEO/AI answers
sitemap.xml · robots.txt · llms.txt
```

## Internal linking rules

- Home hero links to the 4 Tier-1 pages from a "Common jobs" strip.
- Every content page: breadcrumb to home, one CTA block (Download / brew),
  2–3 sideways links to sibling pages, one link back to GitHub.
- Comparison page links to wedge pages, never the reverse-only.
- Language pages all link to /urdu-audio-to-english.html as the flagship
  (it's the one we expect to earn links).

## Page template requirements (every new page)

- Unique title ≤ 60 chars front-loading the query · meta description ≤ 155
- One H1 matching search intent; H2s phrased as questions where natural
- SoftwareApplication JSON-LD reference + page-specific schema (FAQPage on
  /faq, comparison table markup stays plain HTML)
- Canonical tag; OG title/description/image
- ≥ 600 words of genuinely specific content or the page doesn't ship
  (quality gate — no doorway pages)
