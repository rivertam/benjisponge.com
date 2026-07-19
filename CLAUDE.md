# benjisponge.com

Rust SSR personal site on topcoat 0.1.3 — a niche framework; read
`docs/topcoat-notes.md` before writing any topcoat code, don't guess APIs.

## Commands

- `just dev [port]` — live-reload server (default 3000)
- `just build` — cargo build + `topcoat asset bundle`; serving without the bundle step panics
- `just check` — fmt + clippy -D warnings + tests; must pass before claiming done
- `just deploy` — Cloudflare deploy (Worker + container); CI also deploys on push to main. Touching `deploy/` or caching? Read `docs/cloudflare-deploy.md` first

## Adding a page

- Post: `src/app/thoughts/<slug>.rs` + `mod <slug>;` in `thoughts.rs` + entry in `src/content/posts.rs`
- Interest: `src/app/interests/<name>.rs` (copy one; it pulls its copy via `interest("<name>")`) + `mod <name>;` in `interests.rs` + entry in `src/content/interests.rs`
- Other fixed page: also add its route to `src/content/routes.rs::site_routes()`
- Nav, indexes, and 404 all derive from these registries — touch nothing else

## Gotchas

- A `#[page]` module not declared in its parent `mod` silently doesn't route
- Tailwind classes are scanned from `.rs` files at build time; a class rendering unstyled means a stale scan: `touch styles/input.css && cargo build`
- Prose lives in Rust string literals; a `\` continuation eats the newline and the next line's leading spaces, so keep the word-space before the `\`; escape `"`
- `styles/planes-charts.css` hardcodes generated `seg-<bar>-<slice>` and `data-pick-*` names; the css tests in `charts.rs` are the tripwire
- After editing `reference_data.rs` run `just test` — unknown source/option/activity ids panic at render time, tests catch them first
- `?oneway` is presence-only and `trip=oneway` also parses (share-URL back-compat) — don't simplify
- `emissions.rs` deliberately models only the myclimate fuel curve; the missing aircraft-production and infrastructure terms are not an omission to complete
- Units: kg CO₂e and km everywhere; number formatting mirrors Intl.NumberFormat half-away-from-zero — don't "fix" the rounding
