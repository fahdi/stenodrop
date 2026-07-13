# StenoDrop — SEO Strategy

Date: 2026-07-14 · Template: SaaS (adapted for free/open-source desktop app)
Owner: solo (Fahd) · Budget: ~$20/yr (domain) + time · Horizon: 12 months

## Positioning statement (the sentence every page supports)

> StenoDrop is the free Mac app that turns a folder of audio into transcripts —
> offline, private, in 100 languages including Urdu → English.

## Goals & KPIs

Primary conversion = **release download or `git clone`**; secondary = GitHub star.

| Metric | Baseline (Jul 2026) | 3 mo | 6 mo | 12 mo |
|---|---|---|---|---|
| Organic visits/mo | 0 | 150–400 | 500–1,500 | 2,000–5,000 |
| Queries in top 10 (GSC) | 0 | 3–5 long-tail | 10–15 | 25+ incl. 2 comparison terms |
| Indexed pages | 1 | 6–8 | 12–15 | 20–25 |
| Downloads/mo (release + cask) | ~0 | 50 | 250 | 800 |
| GitHub stars | ~0 | 30 | 150 | 500 |
| CWV (mobile) | pass | pass | pass | pass |

Rationale: single-page site on a github.io subpath starts with zero authority;
early wins come from long-tail + distribution channels (Homebrew, awesome
lists, listicles), not head terms.

## Keyword strategy — three tiers

**Tier 1 · Ownable wedges (build pages first)**
- urdu audio to english text / transcribe urdu voice notes / اردو → English
- batch transcribe audio files mac · transcribe a folder of audio files
- transcribe whatsapp voice notes mac
- macwhisper free alternative

**Tier 2 · Category long-tail**
- free offline transcription app mac · transcribe audio to text mac free
- whisper transcription mac app free · transcribe voice memos mac
- punjabi / pashto / hindi / arabic audio to text (one page each, real copy,
  not doorway pages — each names its scripts, dialect caveats, examples)

**Tier 3 · Head terms (do not chase before month 6)**
- audio to text converter free · transcription software mac

## Channel strategy (SEO is only half the traffic)

For a free Mac utility, these outrank blogging in ROI order:
1. **Homebrew cask** (`brew install --cask stenodrop`) — discovery + trust + backlink
2. **awesome-whisper PR** (sindresorhus list) — the canonical directory for this niche
3. **"MacWhisper alternatives" listicles** — email each author; they update yearly
4. **AlternativeTo / MacUpdate / Product Hunt** — profile + launch (PH in month 2–3
   after notarization, not before)
5. **GitHub README as landing page #2** — keyworded title line, screenshots, badges

## E-E-A-T & trust plan

- Notarize releases (Apple Developer ID, $99/yr) by month 3 — every review
  channel penalizes unsigned builds, and "xattr -cr" scares normal users.
- Real screenshots + a 30-second screen recording on the site.
- Maintainer bio with link to fahdmurtaza.com on site + README.
- Public changelog (Releases page is fine) — freshness signal.

## GEO / AI search readiness

- `llms.txt` at site root summarizing what StenoDrop is, requirements, install.
- Q&A-formatted FAQ section ("Is it really free?", "Does audio leave my Mac?",
  "Can it translate Urdu to English?") — passage-level citability for
  AI Overviews / Perplexity / ChatGPT browsing.
- Keep SoftwareApplication JSON-LD current (already shipped); add FAQPage
  schema when FAQ lands.

## Technical foundation (current site)

Already good: single fast static page, JSON-LD, OG tags, mobile-safe, HTTPS.
Gaps to close (roadmap Phase 1): custom domain, sitemap.xml, robots.txt,
llms.txt, OG image (in progress), Search Console + Bing Webmaster verification,
per-page canonicals as pages multiply.

## Risks

| Risk | Mitigation |
|---|---|
| github.io subpath caps authority | Register stenodrop.com/.app NOW; 301 via Pages custom domain |
| Apple ships Urdu in SpeechAnalyzer | Our moat is batch+translate UX, not just language support |
| MacWhisper adds a bigger free tier | Stay the *simplest* free option; speed of iteration |
| Language pages read as doorway spam | Each page gets unique examples, script samples, real caveats |
