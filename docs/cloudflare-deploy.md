# Cloudflare deployment (deploy/)

Architecture: the topcoat binary runs in a Cloudflare Container (it cannot
compile to wasm32 — topcoat-router hard-depends on tokio/hyper), fronted by a
TypeScript Worker (`deploy/src/index.ts`) that owns redirects, edge caching,
and static assets. Config in `deploy/wrangler.jsonc`; image in
`deploy/Dockerfile` (build context = repo root, see `.dockerignore`).

## Request flow

- `/_topcoat/assets/*` — served by the Workers static-asset layer before the
  Worker runs; synced from the image by `just deploy`/CI into
  `deploy/assets/_topcoat/` (gitignored; only `_headers` is committed).
  A missed hash falls through Worker → container, which has the same files on
  disk — stale sync degrades, never breaks.
- GET pages — edge-cached via Cache API (`deploy/src/cache.ts`); cache key
  embeds RELEASE_ID (git sha passed via `--var` at deploy), so each deploy
  atomically invalidates all pages. No purge API calls.
- Non-GET (shard POSTs `/_topcoat/shards/*`) — straight to the container.
- www.benjisponge.com → 301 apex; required, not cosmetic: the planes page
  bakes the Host header into its QR-code URL, so caching two hosts would
  serve wrong QR origins.

## Rules

- Adding dynamic/uncacheable routes (polls etc.): register them in
  `NEVER_CACHE` in `deploy/src/cache.ts`, or add a data-version segment to
  the cache key there. Do not sprinkle cache logic elsewhere.
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

## One-time account setup (manual)

Workers Paid plan; benjisponge.com zone on Cloudflare DNS; GitHub secrets
CLOUDFLARE_API_TOKEN + CLOUDFLARE_ACCOUNT_ID; first deploy from local
(`npx wrangler login` then `just deploy`) to provision the DO migration and
custom domains.
