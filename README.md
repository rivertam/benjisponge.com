# Ben's Site

## Setup

The project uses [proto](https://moonrepo.dev/proto) to pin development tools.
Install the configured toolchain and Git hooks, then start the development server:

```sh
proto use
just install-hooks
just dev
```

The site is available at <http://127.0.0.1:3000>. To use another port:

```sh
just dev 4610
```

`just dev` also starts the local Postgres container and applies migrations.
Seed fitness data separately with `just reset-fitness-local [csv]` (default
`/home/benji/Downloads/WorkoutData.csv`). See `docs/fitness.md`.

## Commands

Run `just` or `just --list` to see the available commands.

```sh
just build
just release
just check
```

## Deploy

- Cloudflare (Worker + container): `just deploy` — see `docs/cloudflare-deploy.md`
- Railway (same Dockerfile): `railway.toml` + `docs/railway-deploy.md`
