# bens-site — design

Personal website for Ben Berman. Server-rendered Rust on
[topcoat](https://docs.rs/topcoat) 0.1.3. No database. One migrated artifact:
the flight page from `~/how-bad`, living on as a blog post with its
calculations moved server-side.

## Goals

1. A home for Ben on the web: who he is, what he's done, what he thinks.
2. **Thoughts** — a freeform blog. Posts are Rust modules, not a CMS: each
   post is arbitrary `view!` content and may bring its own styling, layout,
   even interactivity. The planes page is post #1 and the proof of the
   "freeform" claim.
3. **Experience** — a timeline sourced from LinkedIn (data checked into code).
4. Flight calculations run **server-side** (Rust), not in the browser.
5. **Snapshots** — a script that captures every route as PNGs for visual
   regression + posterity. No unit-test suite (explicit non-goal).

## Stack

- `topcoat 0.1.3`, features `default + tailwind + font-fontsource`.
- Explicit route paths (`#[page("/thoughts")]`) + `Router::builder().discover()`.
  Module-based routing not used — explicit paths keep URLs decoupled from
  module names.
- Tailwind (topcoat's built-in pipeline: `build.rs` +
  `tailwind::stylesheet!()`), self-hosted Fontsource fonts, topcoat assets.
- Client interactivity via topcoat signals/`$()`; server round-trips via
  `#[shard]` (requires `topcoat::runtime::script()` in `<head>`).
- Rust 1.97 (topcoat needs ≥1.95).

## Routes

| Route | Page |
|---|---|
| `/` | Home: hero, latest thoughts, experience digest |
| `/thoughts` | Blog index — dated log of posts |
| `/thoughts/how-bad-are-planes` | The migrated flight page (query params: `from`, `to`, `cabin`, `oneway`, `view`) |
| `/thoughts/pesky-code` | Micro-post: Ben's own Aug-2025 LinkedIn quip (his words verbatim) |
| `/experience` | Timeline: roles, education, skills |

## Module layout

```
src/
├── main.rs                  # tokio main; start(router)
├── app.rs                   # router(), root #[layout] (shell), home #[page]
├── app/
│   ├── thoughts.rs          # /thoughts index
│   ├── thoughts/
│   │   ├── planes.rs        # /thoughts/how-bad-are-planes page + shards
│   │   ├── planes/
│   │   │   ├── form.rs      # flight form + airport combobox (shard)
│   │   │   ├── receipt.rs   # receipt, ice graphic, QR
│   │   │   └── charts.rs    # FlightScale (cuts), ComparisonScale, BudgetChart
│   │   └── pesky_code.rs    # micro-post
│   └── experience.rs        # /experience
├── design.rs                # fonts, palette constants, shared chrome (rail, page_head)
├── content/
│   ├── posts.rs             # post registry: slug, title, date, teaser
│   └── experience.rs        # roles/education/skills data (from LinkedIn)
└── flight/                  # ports of ~/how-bad/src/lib (calculation = server-side)
    ├── emissions.rs         # ← emissions.ts (verbatim model + doc comments)
    ├── airports.rs          # ← airports.ts + metros.ts; data/airports.json embedded
    ├── format.rs            # ← format.ts
    ├── reference_data.rs    # ← reference-data.ts (activities, bars, analogies)
    ├── comparison_scale.rs  # ← comparison-scale.ts
    └── sources.rs           # ← sources.ts
data/airports.json           # copied from ~/how-bad (OurAirports build)
```

`data/airports.json` is embedded with `include_str!` and parsed once into a
`OnceLock` — no DB, no startup I/O.

## Design system

The shell and the migrated post are deliberately two different papers.

**Shell — "mill and oxide".** Ben's headline is "I like Rust"; the site takes
the material seriously. Cool steel-white page, iron ink, and one accent: the
color literally named rust.

- `page #f4f5f7` · `card #ffffff` · `ink #1d2126` · `ink-2 #4c545e` ·
  `muted #8a929c` · `hairline #dde1e6` · `oxide #b7410e` · `oxide-hot #d24a10`
  · `patina #3e7a6c` (used sparingly, e.g. positive/“saved” notes)
- Type (all Fontsource, self-hosted; the Mozilla lineage Rust grew up in):
  **Zilla Slab** 600/700 display · **Fira Sans** 400/500/600 body ·
  **Fira Mono** 400/500 metadata.
- **Signature: the margin rail.** Every page runs a narrow left column of
  Fira Mono metadata — dates on the blog log, year-spans on the timeline,
  section stamps elsewhere — like the stamped margin of an engineering
  logbook. On wide screens it sits in the left gutter; on mobile it collapses
  to inline eyebrows.
- **Oxidation hover:** interactive ink corrodes — links/titles transition
  ink → oxide with the underline thickening. `prefers-reduced-motion`
  respected (no transitions, colors still change).
- Hero on `/` is the three-word bio, huge, in Zilla Slab: “I like Rust.” with
  the subline introducing Ben. No stats row, no gradient.
- Light-only for now (`color-scheme: light`) — matches the migrated post;
  dark is future work.

**Post — its own weather.** `/thoughts/how-bad-are-planes` wraps its article
in the original warm paper (`#f2ead9` page / `#fdf9ef` card / orange `--cost`
`#eb6834` / blue `--save` `#2a78d6`), scoped under a `.paper-warm` wrapper,
with a short new intro in Ben's voice framing it as a migrated artifact. The
original page's CSS vocabulary (receipt, bars, chips) is ported into scoped
styles, not Tailwind-atomized, to keep the port faithful.

## The planes post

Source of truth: `~/how-bad` (`src/App.tsx`, `src/components/*`,
`src/lib/*`, `src/index.css`, plus `docs/superpowers/specs/*` for intent).
One page, one seat's share of a flight, itemized.

Behavior:

- **Form** (from / to / cabin / round-trip) submits **GET** to the same URL —
  `?from=JFK&to=LHR&cabin=business&oneway=1`. URLs stay shareable exactly
  like the original. Server resolves IATA codes → computes `FlightImpact` →
  renders everything. Works with JS disabled.
- **Airport combobox**: progressive enhancement over a plain text input via a
  `#[shard]` (the topcoat shard example is literally a server-backed
  combobox). Typing re-renders suggestions server-side using the ported
  search (`searchAirports`: prefix + one-edit fuzzy + metro aliases).
  Selecting fills the input with the IATA code.
- **Receipt**: whole-aircraft line, hero CO₂e, itemized fuel/contrail/NOx/WTT
  lines, sea-ice row + ice graphic, travel-allowance total, QR code of the
  share URL (`qrcode` crate → PNG → `data:` URI `<img>`; avoids raw-HTML
  injection), route code line. **Dropped from the original:** copy-as-image
  and copy-for-friend clipboard buttons (canvas/clipboard-heavy; the
  shareable URL + QR carry that job). Print-feed animation kept as pure CSS.
- **Charts**: Cuts/Compare tabs toggle client-side (signals; both rendered,
  one hidden — no round trip). FlightScale's cut-chips update a picks signal;
  the affected chart body is a `#[shard]` so the arithmetic stays
  server-side. Zoom panel ("hold them up against the flight") is a signal
  toggle. Tooltips become CSS-only (`.tip` hover/focus bubbles) — no JS
  positioning.
- **BudgetChart** (travel-allowance section) and **IceCallout** ported as
  SSR components.
- **Sources**: cites are superscript links (`#src-<id>`) into a "Sources"
  `<details>`-style section at the foot of the article — same cards, no
  drawer JS.
- **Emissions model** ported line-for-line from `emissions.ts` including its
  long doc comment (myclimate fuel curves, Lee 2021 factors, Teoh 2024 sky
  factor with great-circle sampling, Notz & Stroeve ice, IGES travel budget).
  Parity check: a throwaway binary prints impacts for fixture routes
  (JFK⇄LHR economy RT, SIN→EWR first one-way, SFO⇄LAX economy, LHR⇄SYD
  business) and is compared against the same numbers computed by the original
  TS via `bun` in `~/how-bad`; agreement to 6 significant figures, then the
  binary is deleted (results recorded in the PR/commit message).

## Experience content (from LinkedIn, 2026-07-18)

Roles: DigiChem (Co-Founder/Lead Software Engineer, Aug 2024–present, NYC) ·
Standard Bots (Software Engineer, Sep 2017–Mar 2023, NYC/remote; C++,
TypeScript) · A Plus (Software Engineer, Jun 2015–Dec 2016, NYC) · Wolverine
Trading (SWE Intern, summer 2014, Chicago) · Royal Caribbean (SWE Intern,
summer 2012) · Jackson Memorial Hospital (Research Intern, summer 2009).
Education: Washington University in St. Louis, BS Computer Science 2011–2014,
minor in Design. Skills: Software Design, Robotics, C++, TypeScript (+ Rust,
per the headline). Copy stays factual — titles, dates, places, one plain
sentence each; nothing invented.

## Snapshots (visual regression + posterity)

- `scripts/snapshot [label]`: builds release, starts the server on a free
  port, waits for readiness, then captures every route in a route manifest
  (including a populated flight `?from=JFK&to=LHR` and its
  `view=compare` variant) at 1440px and 390px widths with headless Chrome
  (`google-chrome-stable --headless=new --screenshot`), full page height.
  Output: `snapshots/<YYYY-MM-DD>T<HHMM>__<git-short-sha>[__label]/*.png` +
  `manifest.tsv`; `snapshots/latest` symlink updated. Snapshots are committed.
- `scripts/snapshot-diff <dirA> <dirB>`: ImageMagick `compare -metric AE` per
  matching PNG; writes `diff-*.png` heatmaps and a summary table; exit code
  reflects any pixel drift. Eyeball-driven — no threshold config.

## Non-goals / future

- No database, no auth, no analytics, no comments.
- No unit-test suite (parity + snapshots are the safety net).
- Dark mode, RSS, deployment target: later.
