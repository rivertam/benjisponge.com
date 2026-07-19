# Junk-Drawer Restructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the thoughts/interests split with one flat-URL junk drawer: a single `drawer.rs` registry with `Fun`/`Work` tags, pages at `/{slug}`, and the homepage as the only index.

**Architecture:** One content registry (`src/content/drawer.rs`) replaces `posts.rs` + `interests.rs`; every derived surface (homepage sections, `site_routes()`, 404 suggestions, snapshot manifest) reads it. Page modules flatten from `src/app/{thoughts,interests}/` into `src/app/`; the two index pages are deleted; the nav shrinks to Résumé. No redirects — old URLs 404.

**Tech Stack:** Rust, topcoat 0.1.3 (read `docs/topcoat-notes.md` before writing topcoat code — don't guess APIs), Tailwind (classes scanned from `.rs` files), `just` recipes.

**Spec:** `docs/superpowers/specs/2026-07-18-junk-drawer-restructure-design.md`

## Global Constraints

- `just check` (fmt + clippy `-D warnings` + tests) must pass at the end of every task.
- A `#[page]` module not declared in its parent `mod` silently doesn't route — every moved module needs its `mod` line in `src/app.rs`.
- Prose in Rust string literals: keep the word-space *before* a `\` line continuation; escape `"`.
- `?oneway` presence-only parsing and `trip=oneway` back-compat in the planes page are deliberate — don't touch.
- Number formatting / units in flight code are out of scope — don't touch.
- The "before" snapshot (Task 0) is captured once, before any edits, and never regenerated.
- Commit at the end of every task with the `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer.

---

### Task 0: Pre-change snapshot

**Files:** none modified.

**Interfaces:**
- Produces: `snapshots/<stamp>__<sha>__pre-drawer/` — the "before" for Task 4's diff.

- [ ] **Step 1: Confirm clean tree** — `git status` must be clean (the snapshot script stamps the SHA; uncommitted edits poison it).

- [ ] **Step 2: Capture**

Run:
```bash
just release && just snapshot pre-drawer
```
Expected: PNGs for all 15 current routes land in a new `snapshots/…__pre-drawer/` directory; `snapshots/latest` updates. No commit (snapshots are artifacts).

---

### Task 1: The drawer registry

**Files:**
- Create: `src/content/drawer.rs`
- Modify: `src/content.rs` (add `pub mod drawer;` — keep `posts`/`interests` for now; they die in Task 3)

**Interfaces:**
- Produces: `pub enum Tag { Fun, Work }` (derives `Clone, Copy, PartialEq, Eq, Debug`); `pub struct DrawerPage { slug, title, teaser, tags: &'static [Tag], date: Option<&'static str> }`; `pub static PAGES: [DrawerPage; 10]`; `pub fn drawer_page(slug: &str) -> &'static DrawerPage` (panics on unknown slug, same contract as the old `interest()`).

- [ ] **Step 1: Write `src/content/drawer.rs`** — registry + invariant tests in one file:

```rust
//! The junk drawer: the single registry behind every non-fixed page. Each
//! entry is a standalone page module in `src/app/`; this list is the single
//! source of truth for its slug (route `/{slug}`), display title, teaser
//! (homepage card copy, doubling as the page's lede), tags (which homepage
//! sections it files under), and optional date (the rail stamp). The
//! homepage, the 404's route list, and the snapshot manifest all derive from
//! here — adding a page means one entry here plus the page module.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tag {
    Fun,
    Work,
}

pub struct DrawerPage {
    pub slug: &'static str,
    pub title: &'static str,
    pub teaser: &'static str,
    /// At least one; a page may carry both and files under both sections.
    pub tags: &'static [Tag],
    /// Shown as the card's rail stamp when present.
    pub date: Option<&'static str>,
}

pub static PAGES: [DrawerPage; 10] = [
    DrawerPage {
        slug: "how-bad-are-planes",
        title: "How bad are planes?",
        teaser: "The part of the fare you don't pay.",
        tags: &[Tag::Fun],
        date: Some("2026-07-12"),
    },
    DrawerPage {
        slug: "pesky-code",
        title: "Pesky code",
        teaser: "On what AI frees me up to truly love.",
        tags: &[Tag::Fun],
        date: Some("2025-08-14"),
    },
    DrawerPage {
        slug: "drums",
        title: "Drums",
        teaser: "Mediocre drummer. Recording turns out to be much harder than playing.",
        tags: &[Tag::Fun],
        date: None,
    },
    DrawerPage {
        slug: "swing",
        title: "Swing dancing",
        teaser: "Swing dancing (lead and follow but mostly lead)",
        tags: &[Tag::Fun],
        date: None,
    },
    DrawerPage {
        slug: "lifting",
        title: "Lifting",
        teaser: "Deadlift PR 345 lbs, Squat PR 235 lbs, Bench PR like 165 but I never 1RM it",
        tags: &[Tag::Fun],
        date: None,
    },
    DrawerPage {
        slug: "keys",
        title: "Keyboards",
        teaser: "Split-columnar keyboard person. Ten thousand strangers have watched my Dactyl Manuform video; TypeRacer has me at 117wpm.",
        tags: &[Tag::Fun],
        date: None,
    },
    DrawerPage {
        slug: "spire",
        title: "Slay the Spire",
        teaser: "Slay the Spire at Ascension 20, with an annotated run synopsis, because a win nobody can audit barely counts.",
        tags: &[Tag::Fun],
        date: None,
    },
    DrawerPage {
        slug: "models",
        title: "Toy models",
        teaser: "Procedural cities with opinionated residents — a react-three-fiber toy running Schelling-style agents.",
        tags: &[Tag::Fun],
        date: None,
    },
    DrawerPage {
        slug: "puzzles",
        title: "Crosswords",
        teaser: "A Rust crossword engine, so .puz files open in the terminal. Nobody had asked for this.",
        tags: &[Tag::Fun],
        date: None,
    },
    DrawerPage {
        slug: "felix",
        title: "Felix",
        teaser: "There is a dog. There is, accordingly, a website computing when we are the same age.",
        tags: &[Tag::Fun],
        date: None,
    },
];

/// A page's own registry entry. Panics on a slug not in the registry —
/// pages pass literals, and every page renders in the snapshot run, so a
/// typo surfaces on the first capture.
pub fn drawer_page(slug: &str) -> &'static DrawerPage {
    PAGES
        .iter()
        .find(|p| p.slug == slug)
        .unwrap_or_else(|| panic!("unknown drawer slug: {slug}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Slugs the fixed routes claim; a drawer slug matching one would
    /// shadow or collide at the router.
    const FIXED_SEGMENTS: [&str; 1] = ["resume"];

    #[test]
    fn slugs_are_unique_and_urlish() {
        for (i, page) in PAGES.iter().enumerate() {
            assert!(
                page.slug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "slug {:?} is not lowercase-kebab",
                page.slug
            );
            assert!(
                !PAGES[i + 1..].iter().any(|other| other.slug == page.slug),
                "duplicate slug {:?}",
                page.slug
            );
            assert!(
                !FIXED_SEGMENTS.contains(&page.slug),
                "slug {:?} collides with a fixed route",
                page.slug
            );
        }
    }

    #[test]
    fn every_page_has_copy_and_tags() {
        for page in PAGES.iter() {
            assert!(!page.title.is_empty(), "{}: empty title", page.slug);
            assert!(!page.teaser.is_empty(), "{}: empty teaser", page.slug);
            assert!(!page.tags.is_empty(), "{}: no tags", page.slug);
            for tag in [Tag::Fun, Tag::Work] {
                assert!(
                    page.tags.iter().filter(|t| **t == tag).count() <= 1,
                    "{}: duplicate tag {:?}",
                    page.slug,
                    tag
                );
            }
        }
    }

    #[test]
    fn dates_are_iso() {
        for page in PAGES.iter() {
            if let Some(date) = page.date {
                let ok = date.len() == 10
                    && date.chars().enumerate().all(|(i, c)| match i {
                        4 | 7 => c == '-',
                        _ => c.is_ascii_digit(),
                    });
                assert!(ok, "{}: date {:?} is not YYYY-MM-DD", page.slug, date);
            }
        }
    }

    #[test]
    fn lookup_panics_on_unknown_slug() {
        assert!(std::panic::catch_unwind(|| drawer_page("no-such-page")).is_err());
        assert_eq!(drawer_page("drums").slug, "drums");
    }
}
```

- [ ] **Step 2: Register the module** — in `src/content.rs`, add `pub mod drawer;` (alphabetical, so first):

```rust
pub mod drawer;
pub mod experience;
pub mod interests;
pub mod patches;
pub mod posts;
pub mod routes;
```

- [ ] **Step 3: Run the tests**

Run: `cargo test drawer`
Expected: 4 tests PASS. (If `slugs_are_unique_and_urlish` fails, a registry entry was mistyped — fix the data, not the test.)

- [ ] **Step 4: Full check** — `just check`. Expected: clean (nothing consumes the new module yet; `dead_code` doesn't fire on `pub` items in a binary's reachable module tree — if clippy complains about unused items, silence is NOT the fix; it means Step 2 was missed).

- [ ] **Step 5: Commit**

```bash
git add src/content/drawer.rs src/content.rs
git commit -m "Content: drawer registry with Fun/Work tags

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Flatten the page modules to `/{slug}`

**Files:**
- Move: `src/app/thoughts/pesky_code.rs` → `src/app/pesky_code.rs`; `src/app/thoughts/planes.rs` → `src/app/planes.rs`; `src/app/thoughts/planes/` → `src/app/planes/`; all 8 of `src/app/interests/*.rs` → `src/app/*.rs`
- Delete: `src/app/thoughts.rs`, `src/app/interests.rs` (the index pages)
- Modify: `src/app.rs` (mod declarations), `src/app/planes.rs` (route + share URL), `src/app/planes/form.rs` (eyebrow link), each moved interest page (route, lookup, back-link)

**Interfaces:**
- Consumes: `crate::content::drawer::drawer_page` from Task 1.
- Produces: routes `/pesky-code`, `/how-bad-are-planes`, `/drums`, `/swing`, `/lifting`, `/keys`, `/spire`, `/models`, `/puzzles`, `/felix`. Module names in `src/app.rs`: `pesky_code`, `planes`, `drums`, `swing`, `lifting`, `keys`, `spire`, `models`, `puzzles`, `felix`.

Transient state note: after this task the nav and homepage still point at `/thoughts*` / `/interests*` (now 404s) and `routes.rs` still lists the old paths. That's expected; Task 3 fixes all derived surfaces. Code compiles and tests pass throughout.

- [ ] **Step 1: Move the files**

```bash
git mv src/app/thoughts/pesky_code.rs src/app/pesky_code.rs
git mv src/app/thoughts/planes.rs src/app/planes.rs
git mv src/app/thoughts/planes src/app/planes
git mv src/app/interests/drums.rs src/app/interests/felix.rs src/app/interests/keys.rs src/app/interests/lifting.rs src/app/interests/models.rs src/app/interests/puzzles.rs src/app/interests/spire.rs src/app/interests/swing.rs src/app/
git rm src/app/thoughts.rs src/app/interests.rs
rmdir src/app/thoughts src/app/interests 2>/dev/null || true
```

- [ ] **Step 2: Re-declare the modules** — `src/app.rs` top block becomes (order alphabetical; every one of these MUST appear or its page silently doesn't route):

```rust
mod drums;
mod felix;
mod keys;
mod lifting;
mod models;
mod not_found;
mod pesky_code;
mod planes;
mod puzzles;
mod resume;
mod spire;
mod swing;
```

(The old `mod pesky_code; mod planes;` lines lived in the deleted `thoughts.rs`, and the 8 interest `mod`s in the deleted `interests.rs` — they all move here.)

- [ ] **Step 3: Update `src/app/pesky_code.rs`** — one line:

`#[page("/thoughts/pesky-code")]` → `#[page("/pesky-code")]`

- [ ] **Step 4: Update `src/app/planes.rs`** — two edits:

`#[page("/thoughts/how-bad-are-planes")]` → `#[page("/how-bad-are-planes")]`

and the share-URL builder (~line 82):

```rust
let mut share_path = format!(
    "/how-bad-are-planes?from={}&to={}",
    from.iata, to.iata
);
```

Leave the `?oneway` / `trip=oneway` handling and everything else alone.

- [ ] **Step 5: Update `src/app/planes/form.rs`** — the eyebrow (~line 34) linked to the dead thoughts index; point it home:

```rust
<p class="eyebrow">
    <a href="/">"ben berman"</a>
    (if revealed { " · how bad are planes" } else { " / how bad are planes" })
</p>
```

- [ ] **Step 6: Update each of the 8 interest pages** (`drums.rs`, `felix.rs`, `keys.rs`, `lifting.rs`, `models.rs`, `puzzles.rs`, `spire.rs`, `swing.rs` — now in `src/app/`). Exactly four mechanical edits per file, shown here for `drums.rs`; apply identically (with the file's own slug) to all 8:

1. Import: `content::interests::interest` → `content::drawer::drawer_page`
2. Route: `#[page("/interests/drums")]` → `#[page("/drums")]`
3. Lookup: `let meta = interest("drums");` → `let meta = drawer_page("drums");`
4. Back-link footer: `<a class="quiet-link" href="/interests">"← all interests"</a>` → `<a class="quiet-link" href="/">"← the drawer"</a>`

- [ ] **Step 7: Verify it compiles and routes exist**

Run: `just check`
Expected: clean. Then prove routing (registry-derived routes still say `/thoughts/*` — check the actual router instead):

```bash
cargo build && topcoat asset bundle
PORT=3111 ./target/debug/bens-site &
sleep 1
curl -s -o /dev/null -w '%{http_code} ' http://localhost:3111/drums http://localhost:3111/pesky-code http://localhost:3111/how-bad-are-planes http://localhost:3111/interests; echo
kill %1
```
Expected output: `200 200 200 404`

- [ ] **Step 8: Commit**

```bash
git add -A src/app.rs src/app
git commit -m "Routes: flatten pages to /{slug}, drop index pages

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Derived surfaces — homepage drawer, nav, routes; delete old registries

**Files:**
- Modify: `src/app.rs` (homepage), `src/design.rs` (nav), `src/content/routes.rs`, `src/content.rs`
- Delete: `src/content/posts.rs`, `src/content/interests.rs`
- Modify: `styles/site.css` (remove dead `.nav-dd` rules, ~lines 148–190)

**Interfaces:**
- Consumes: `PAGES`, `Tag`, `DrawerPage` from `crate::content::drawer` (Task 1); flat routes from Task 2.
- Produces: `site_routes()` = `["/", "/resume", "/{slug}"×10]` — 12 routes total.

- [ ] **Step 1: Re-point `src/content/routes.rs`**

```rust
//! The site's canonical route list, derived from the drawer registry. The
//! 404's suggestion index and the snapshot manifest (`bens-site --routes`)
//! both read this, so a page that exists here but not in the router — or
//! vice versa — is a bug in exactly one place.

use crate::content::drawer::PAGES;

/// Every real route on the site, fixed pages first.
pub fn site_routes() -> Vec<String> {
    let mut routes = vec!["/".to_string(), "/resume".to_string()];
    routes.extend(PAGES.iter().map(|p| format!("/{}", p.slug)));
    routes
}

/// A route's snapshot file stem: "/" is "home"; any other path drops the
/// leading slash and dashes the rest ("/pesky-code" → "pesky-code").
pub fn route_name(route: &str) -> String {
    if route == "/" {
        "home".to_string()
    } else {
        route.trim_start_matches('/').replace('/', "-")
    }
}
```

- [ ] **Step 2: Rebuild the homepage as the drawer** — in `src/app.rs`, keep the hero section verbatim, replace the Thoughts section with Work + Fun sections, and add a card component. Replace the `use crate::{content::posts::POSTS, design::shell};` import with:

```rust
use crate::{
    content::drawer::{DrawerPage, PAGES, Tag},
    design::shell,
};
```

Add above `home` (component syntax per `docs/topcoat-notes.md` — check it, don't guess):

```rust
/// One drawer card: date (or nothing) in the rail, title, teaser.
#[component]
async fn drawer_card(entry: &'static DrawerPage) -> Result {
    view! {
        <article class="rail-row">
            <p class="rail-stamp">(entry.date.unwrap_or(""))</p>
            <div class="min-w-0">
                <h3 class="font-display text-2xl leading-snug font-semibold">
                    <a class="oxlink" href=(format!("/{}", entry.slug))>(entry.title)</a>
                </h3>
                <p class="mt-1.5 max-w-prose text-ink2">(entry.teaser)</p>
            </div>
        </article>
    }
}
```

(`#[component]` and `View` need importing: `use topcoat::view::{component, view};` — merge into the existing `topcoat::` use tree.)

Then the page body after the hero:

```rust
#[page("/")]
async fn home() -> Result {
    let work: Vec<&'static DrawerPage> =
        PAGES.iter().filter(|p| p.tags.contains(&Tag::Work)).collect();
    let fun: Vec<&'static DrawerPage> =
        PAGES.iter().filter(|p| p.tags.contains(&Tag::Fun)).collect();
    let body = view! {
        // Hero: the three-word bio, huge. "Rust." takes the accent — it is,
        // after all, the color the palette is named for.
        <section class="mt-20 sm:mt-28">
            <h1 class="font-display text-[3.4rem] leading-[0.95] font-bold tracking-tight sm:text-[5.5rem]">
                "I like "
                <span class="text-oxide">"Rust."</span>
            </h1>
            <p class="mt-6 max-w-prose text-lg text-ink2">
                "I'm a software engineer in New York; co-founder of DigiChem; \
                 this site is where I think out loud."
            </p>
        </section>

        // The drawer. Work first — it's short — then the pile.
        <section class="mt-24 space-y-10 border-t border-hairline pt-8">
            <div class="rail-row">
                <h2 class="rail-stamp uppercase tracking-[0.18em]">"Work"</h2>
                <div></div>
            </div>
            <article class="rail-row">
                <p class="rail-stamp"></p>
                <div class="min-w-0">
                    <h3 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/resume">"Résumé"</a>
                    </h3>
                    <p class="mt-1.5 max-w-prose text-ink2">"The professional record."</p>
                </div>
            </article>
            for entry in work {
                drawer_card(entry: entry)
            }
        </section>

        <section class="mt-16 space-y-10 border-t border-hairline pt-8">
            <div class="rail-row">
                <h2 class="rail-stamp uppercase tracking-[0.18em]">"Fun"</h2>
                <div></div>
            </div>
            for entry in fun {
                drawer_card(entry: entry)
            }
        </section>
    }?;
    view! { shell(title: "Ben Berman", body: body) }
}
```

(Copy note: "The professional record." and "← the drawer" (Task 2) are placeholder-grade copy Ben may want to rewrite — flag in the final report, don't block on it.)

- [ ] **Step 3: Shrink the nav** — in `src/design.rs`: delete `use crate::content::interests::INTERESTS;` and replace the whole `<nav>…</nav>` block (thoughts link + interests `<details>` dropdown) with:

```rust
<nav class="flex gap-6 font-meta text-sm">
    <a href="/resume" class="quiet-link">"résumé"</a>
</nav>
```

- [ ] **Step 4: Delete the old registries**

```bash
git rm src/content/posts.rs src/content/interests.rs
```

and in `src/content.rs` drop their `mod` lines:

```rust
pub mod drawer;
pub mod experience;
pub mod patches;
pub mod routes;
```

- [ ] **Step 5: Remove dead nav CSS** — in `styles/site.css`, delete every `.nav-dd` rule block (`.nav-dd`, `.nav-dd > summary`, `.nav-dd > summary::-webkit-details-marker`, `.nav-dd > summary::after`, `.nav-dd[open] > summary::after`, `.nav-dd-menu`, `.nav-dd-menu > a` — roughly lines 148–190). Touch nothing else in the file.

- [ ] **Step 6: Full check with fresh Tailwind scan** (moved/edited `.rs` files can leave a stale class scan):

Run: `touch styles/input.css && cargo build && just check`
Expected: clean build, clippy clean, all tests pass (drawer tests + flight core + charts CSS tripwire).

- [ ] **Step 7: Verify the route manifest**

Run: `./target/debug/bens-site --routes`
Expected: exactly 12 lines — `/	home`, `/resume	resume`, then the 10 drawer slugs, no `/thoughts*` or `/interests*`.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "Homepage is the drawer: Work/Fun sections, slim nav, one registry

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Visual verification

**Files:** none (verification only; fix-ups if the diff reveals breakage).

**Interfaces:**
- Consumes: Task 0's `…__pre-drawer/` snapshot directory.

- [ ] **Step 1: Capture "after"**

```bash
just release && just snapshot post-drawer
```

- [ ] **Step 2: Diff against Task 0's capture**

```bash
just snapshot-diff <pre-drawer-dir-name> <post-drawer-dir-name>
```

Expected: every route name changed, so most of the diff is adds/removes by design. Actually inspect: `home@*.png` (hero unchanged, Work + Fun sections render, dated cards show stamps, undated cards show empty rail), `resume@*.png` (only the nav should differ), and spot-check `drums@*.png` + `how-bad-are-planes@*.png` against their old `interests-drums@*.png` / `thoughts-how-bad-are-planes@*.png` (body identical; only nav + back-link/eyebrow differ). Read the PNGs — don't assert from the exit code alone.

- [ ] **Step 3: Interactive sanity check on planes** — the share URL and form must still round-trip:

```bash
PORT=3111 ./target/release/bens-site &
sleep 1
curl -s -o /dev/null -w '%{http_code}\n' 'http://localhost:3111/how-bad-are-planes?from=JFK&to=LHR&trip=oneway'
curl -s 'http://localhost:3111/how-bad-are-planes?from=JFK&to=LHR' | grep -o '/how-bad-are-planes?from=JFK&amp;to=LHR' | head -1
kill %1
```
Expected: `200`, and the share link in the page body carries the new flat path.

- [ ] **Step 4: Final gate** — `just check` one last time; report results (including the two placeholder copy strings from Task 3) honestly.
