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

`just dev` also starts the local fitness API on port 8791 and idempotently
imports `/home/benji/Downloads/WorkoutData.csv`. Override its path with
`WORKOUT_DATA_CSV=/path/export.csv just dev`. See `docs/fitness.md`.

## Commands

Run `just` or `just --list` to see the available commands.

```sh
just build
just release
just check
```
