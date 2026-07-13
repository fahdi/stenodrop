# StenoDrop — SEO Implementation Roadmap

## Phase 1 · Foundation (weeks 1–4) — mostly one sitting each

| # | Task | Effort | Depends on |
|---|---|---|---|
| 1 | Register stenodrop.com (+ .app if cheap) | 10 min + $ | Fahd only |
| 2 | Custom domain on Pages (CNAME, HTTPS, www→apex) | 30 min | 1 |
| 3 | OG image (in progress this session) | done | — |
| 4 | sitemap.xml + robots.txt + llms.txt | 30 min | — |
| 5 | Google Search Console + Bing Webmaster verify, submit sitemap | 30 min | 2 ideally |
| 6 | GitHub README SEO pass (title line, badges, screenshots, topics) | 1 hr | — |
| 7 | Homebrew cask submission | 2 hr | stable release tag |
| 8 | awesome-whisper PR + AlternativeTo listing | 1 hr | — |

Exit criteria: site on custom domain, indexed, discoverable via brew.

## Phase 2 · Expansion (weeks 5–12)

- Ship the four Tier-1 pages + FAQ (see CONTENT-CALENDAR.md)
- Apple Developer ID → notarized 0.2.0; remove xattr instructions everywhere
- Product Hunt launch once notarized
- Listicle outreach round 1; monitor GSC weekly, note first impressions

Exit criteria: 6–8 indexed pages, first non-brand clicks in GSC, notarized build.

## Phase 3 · Scale (weeks 13–24)

- Language cluster pages (unique content each, quality gate enforced)
- YouTube demo (embeds + a second SERP surface)
- Outreach round 2; r/macapps, MacUpdate
- Quarterly refresh of Tier-1 pages tied to Whisper model releases
- GSC-driven: expand whatever shows impressions, prune what doesn't

Exit criteria: 500+ organic visits/mo, 10+ top-10 queries, 250+ downloads/mo.

## Phase 4 · Authority (months 7–12)

- Engineering write-up → HN/Lobsters (links, not keywords)
- Press pitches to Mac blogs with traction numbers
- Evaluate App Store listing as parallel channel
- Evaluate paid "Pro" tier only if downloads justify it (diarization/SRT
  would be the obvious Pro features — exactly MacWhisper's paid ground)

## Measurement ritual (15 min, every Monday)

1. GSC: new queries with impressions → candidates for content or title tweaks
2. GitHub Insights: traffic sources → which channel actually converts
3. Release download counts (gh api) → the real KPI
4. One action item max per week; this is a solo project, protect the time.

## Explicit non-goals for year 1

Paid ads, link buying, multi-language site UI, blog-for-blogging's-sake,
chasing "audio to text converter" head terms.
