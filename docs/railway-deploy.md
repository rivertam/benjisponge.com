# Railway deployment

The site binary runs as a Railway service built from `deploy/Dockerfile`
(same multi-stage image as Cloudflare Containers). Config is in
`railway.toml` at the repo root — it forces the Dockerfile builder so
Railpack/Nixpacks cannot fall back to an old `rustc` and a bare
`cargo build --release` (which skips `topcoat asset bundle` and panics at
runtime without `assets/manifest.toml`).

## Build

- Builder: Dockerfile at `deploy/Dockerfile`, context = repo root.
- Image: `rust:1.97-slim` build → `debian:trixie-slim` runtime; installs
  `topcoat-cli`, runs `cargo build --release`, then
  `topcoat asset bundle --release --bin benjisponge`.
- Do not set a custom build command. Do not use Railpack for this service.
- Confirm the build log says it is using the Dockerfile. If it still runs
  Railpack, set service variable `RAILWAY_DOCKERFILE_PATH=deploy/Dockerfile`
  (or `NO_CACHE=1` once) and redeploy.

`HOST=0.0.0.0` is baked into the image; Railway injects `PORT` at runtime.

## Database and variables

Add a Railway Postgres plugin (or point at an existing Postgres). The app
reads `POSTGRES_URL` only — not `DATABASE_URL`. On the web service:

```
POSTGRES_URL=${{Postgres.DATABASE_URL}}
```

Also set (same names as Cloudflare Worker secrets):

- `SPIRE_SYNC_TOKEN` — Bearer for `POST /api/spire/runs`
- `FITNESS_SYNC_TOKEN` — Bearer for fitness import
- `SITE_ORIGIN` — optional; absolute links / analytics origin checks.
  Defaults to `https://benjisponge.com` when unset.

Unset sync tokens close those write endpoints. Missing `POSTGRES_URL`
leaves data pages degraded but the process still starts.

## Migrations

Schema lives in `toasty/migrations/`. The runtime image does not ship the
`migrate` binary — apply from a machine with the repo and
`POSTGRES_URL` pointed at Railway:

```sh
POSTGRES_URL='postgresql://...' just migrate migration apply
```

Or put Railway's URL in `.env` as `POSTGRES_URL` and run
`just migrate migration apply`. `just dev` only migrates local Docker
Postgres; it does not touch Railway.

After the first successful migrate, normal deploys only need a new image
when code or assets change. New migrations still need an explicit apply
before (or right after) the deploy that depends on them.

## Sync CLIs

`just sync-spire` and `just sync-fitness` POST to the live site. Point
them at the Railway public URL (or custom domain) the same way as
production today; tokens must match the service variables above. Details:
`docs/cloudflare-deploy.md` (spire) and `docs/fitness.md` (fitness).

## Cutover notes

- Custom domain: attach in Railway and update DNS; keep apex vs `www`
  consistent — planes QR codes bake the request Host.
- Cloudflare edge cache / Worker assets are not used on Railway; the
  container serves HTML and `/_topcoat/assets/*` itself.
- Rotating `POSTGRES_URL` or sync tokens requires a redeploy/restart so
  the process picks up the new env.
