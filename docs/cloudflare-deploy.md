# Cloudflare deployment (deploy/)

Architecture: the topcoat binary runs in a Cloudflare Container (it cannot
compile to wasm32 — topcoat-router hard-depends on tokio/hyper), fronted by a
TypeScript Worker (`deploy/src/index.ts`) that owns redirects, edge caching,
and static assets — nothing else. Every application endpoint, pages and
`/api/*` alike, is served by the Rust app inside the container, which talks to
Postgres via the `POSTGRES_URL` secret. Config in `deploy/wrangler.jsonc`;
image in `deploy/Dockerfile` (build context = repo root, see `.dockerignore`).

## Request flow

- `/_topcoat/assets/*` — served by the Workers static-asset layer before the
  Worker runs; synced from the image by `just deploy`/CI into
  `deploy/assets/_topcoat/` (gitignored; only `_headers` is committed).
  A missed hash falls through Worker → container, which has the same files on
  disk — stale sync degrades, never breaks.
- GET pages — eligible for edge caching via Cache API (`deploy/src/cache.ts`),
  unless the origin returns `Cache-Control: no-store` or `private`; cache keys
  embed RELEASE_ID (git sha passed via `--var` at deploy), so each deploy
  atomically invalidates all stored pages. No purge API calls.
- `/api/*` and non-GET requests — straight to the container, never
  edge-cached.
- www.benjisponge.com → 301 apex; required, not cosmetic: the planes page
  bakes the Host header into its QR-code URL, so caching two hosts would
  serve wrong QR origins.

## Rules

- Deploy from a committed HEAD. RELEASE_ID is `git rev-parse --short HEAD`,
  so deploying uncommitted changes reuses the previous cache keys and the
  edge keeps serving the old HTML (and a warm container may keep serving the
  old image until it restarts). Pages that set their own `s-maxage` (the
  spire-rendering pages use 60 s) self-heal on that timescale; anything else
  needs a commit.
- Adding dynamic/uncacheable routes (polls etc.): return a
  `Cache-Control: no-store` or `Cache-Control: private` header from the
  renderer. For dynamic responses that should remain cached but refresh
  between deploys, set an explicit `s-maxage` instead (see how the spire
  pages do it in `src/app.rs`). Do not sprinkle route-specific cache logic
  in the Worker.
- `manifest.toml` is deliberately excluded from the synced assets: it is
  unhashed and only the binary needs it (its own copy ships in the image).
- After editing `wrangler.jsonc`: rerun `npx wrangler types` in `deploy/`
  (regenerates `worker-configuration.d.ts`), then `npx tsc`.
- RELEASE_ID has a `"dev"` placeholder in `vars` so types/`wrangler dev`
  work; real deploys override it — keep passing `--var RELEASE_ID:<sha>`.
- The Dockerfile's `ENV CARGO_HOME=/root/.cargo` is load-bearing: topcoat-cli
  strips `CARGO_HOME` before the internal `cargo build` that
  `topcoat asset bundle` runs, and the rust image's default
  (`/usr/local/cargo`) would make that inner build a full recompile instead
  of a cache hit. Don't remove it.
- If the `topcoat asset bundle` build step sits silent for minutes, it is
  almost certainly downloading its ~31 fontsource woff2 files serially with
  broken container DNS (an unreachable first nameserver costs a 5s timeout
  per lookup). Verify with a `curl -w '%{time_namelookup}'` inside any
  container; fix DNS at the docker daemon/host level, not in this repo.
- Local deploys and `wrangler dev` need the docker buildx plugin
  (`pacman -S docker-buildx`); CI sets up buildx itself.
- Container scales to zero (`sleepAfter = "15m"`); 1–3 s cold start on first
  uncached/POST hit after idle is accepted — don't add cron-warming without
  asking.

## Database and secrets

The site's data (spire runs, fitness archive) lives in one Postgres database.
The Worker never touches it: `POSTGRES_URL`, `SPIRE_SYNC_TOKEN`, and
`FITNESS_SYNC_TOKEN` are Worker secrets whose only job is to be forwarded
into the container process via `envVars` in `deploy/src/index.ts`. That
channel is read at instance start — rotating a secret needs a container
restart. Unset secrets become empty strings, which the Rust side treats as
"closed" (auth) or "unconfigured" (database).

- Schema is managed by toasty migrations (`toasty/migrations/`). Apply to
  production with `just migrate migration apply` (reads `POSTGRES_URL` from
  `.env`); `just dev` applies them to the local Postgres automatically.
- Install or rotate secrets from `deploy/`:

  ```sh
  npx wrangler secret put POSTGRES_URL
  npx wrangler secret put SPIRE_SYNC_TOKEN < ~/.config/benjisponge/spire.token
  npx wrangler secret put FITNESS_SYNC_TOKEN < ~/.config/benjisponge/fitness.token
  ```

  Regenerate a token file with `openssl rand -hex 32 > <path>` before the
  matching `secret put`.

### Spire runs

- API in `src/app/interests/spire/api.rs`: `GET /api/spire/runs` and
  `GET /api/spire/ids` are public; `POST /api/spire/runs` needs the
  `SPIRE_SYNC_TOKEN` Bearer token. The local copy the CLI reads lives at
  `~/.config/benjisponge/spire.token`.
- Write path is `just sync-spire` (`src/app/interests/spire/spire_sync.rs`)
  on the machine
  with the save files. Idempotent twice over: the CLI diffs against
  `/api/spire/ids`, and the server skips already-stored run ids.
- The container renders `/spire`, `/`, and `/feed.xml` from the run table
  through a 60 s in-process cache (`src/app/interests/spire/runs.rs`; the
  import
  endpoint drops it immediately on write), and those pages return
  `s-maxage=60` — so a sync is visible within a minute with no deploy and no
  purge call.

### Fitness archive

Everything — cross-layer map, source invariants, API/filter/import contracts,
taxonomy workflow, local and production reset procedures — lives in
`docs/fitness.md`.

## One-time account setup (manual)

Workers Paid plan; benjisponge.com zone on Cloudflare DNS; GitHub secrets
CLOUDFLARE_API_TOKEN + CLOUDFLARE_ACCOUNT_ID; first deploy from local
(`npx wrangler login` then `just deploy`) to provision the DO migration and
custom domains.
