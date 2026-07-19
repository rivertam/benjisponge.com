# Junk-drawer restructure — design

2026-07-18

## Goal

Replace the thoughts/interests split with a single "junk drawer": one flat
top-level URL per page, one registry, two categories (**Fun** and **Work**) as
metadata, and the homepage as the only index.

No redirects from old URLs — `/thoughts/*` and `/interests/*` simply 404.

## Registry

New `src/content/drawer.rs` replaces `posts.rs` and `interests.rs`:

```rust
pub enum Kind { Fun, Work }

pub struct DrawerPage {
    pub slug: &'static str,      // route is /{slug}
    pub title: &'static str,
    pub teaser: &'static str,
    pub kind: Kind,
    pub date: Option<&'static str>, // shown as the rail stamp when present
}

pub static PAGES: [DrawerPage; 10] = [ /* ... */ ];
```

- Array order = display order within each kind's homepage section.
- `drawer_page(slug)` lookup helper replaces `interest(slug)`, same
  panic-on-unknown-slug contract (snapshot run surfaces typos).
- Contents: the 8 hobby pages (Fun, no date), `pesky-code` (Fun, dated),
  `how-bad-are-planes` (Fun, dated). Resume stays a fixed page, listed in the
  Work section of the homepage but not a registry entry (it has no teaser
  card semantics today; revisit if more work pages appear).

## Routing & modules

- Page modules flatten: `src/app/thoughts/*` and `src/app/interests/*` move
  into `src/app/` (planes keeps its submodule tree under `src/app/planes/`).
  Each page's `#[page]` route becomes `/{slug}`; remember the parent-`mod`
  declaration gotcha.
- Delete `src/app/thoughts.rs` and `src/app/interests.rs` (both index pages
  and their `mod` declarations). No redirect handlers.
- `routes.rs::site_routes()` = `/`, `/resume`, plus one route per `PAGES`
  entry.

## Homepage

Keeps the hero, then becomes the drawer, reusing the existing `rail-row`
card layout:

1. **Work** section: resume link (+ future Work-kind pages).
2. **Fun** section: all Fun-kind pages; dated entries show the date in the
   rail stamp, undated entries leave it empty.

Nav in `design.rs::shell` drops the Thoughts/Interests entries (including
any dropdown) and keeps Home and Resume.

## Tests & tooling

- Data-invariant tests re-point at `drawer.rs`: unique slugs, non-empty
  fields, slugs don't collide with fixed routes (`resume`), every registry
  entry has a routed page (via `site_routes()` ↔ router parity, as today).
- Snapshot manifest derives from the new routes automatically. Capture a
  pre-change snapshot before touching anything; expect the diff to be
  rename-heavy since every route name changes.
- `just check` green before done.

## Out of scope

- Redirects / URL back-compat of any kind.
- New content or copy changes beyond what the merge forces.
- Recategorizing pages beyond Fun/Work as specified.
