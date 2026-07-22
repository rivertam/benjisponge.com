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
- GET pages — eligible for edge caching via Cache API (`deploy/src/cache.ts`),
  unless the origin returns `Cache-Control: no-store` or `private`; cache keys
  embed RELEASE_ID (git sha passed via `--var` at deploy), so each deploy
  atomically invalidates all stored pages. No purge API calls.
- Non-GET (shard POSTs `/_topcoat/shards/*`) — straight to the container.
- www.benjisponge.com → 301 apex; required, not cosmetic: the planes page
  bakes the Host header into its QR-code URL, so caching two hosts would
  serve wrong QR origins.

## Rules

- Deploy from a committed HEAD. RELEASE_ID is `git rev-parse --short HEAD`,
  so deploying uncommitted changes reuses the previous cache keys and the
  edge keeps serving the old HTML (and a warm container may keep serving the
  old image until it restarts). Escape hatch if it happens: bump the spire
  data version (`npx wrangler d1 execute benjisponge-spire --remote
  --command "UPDATE spire_meta SET v = v + 1 WHERE k = 'version'"`) — that
  re-keys `/`, `/spire`, and `/feed.xml`; anything else needs a commit.
- Adding dynamic/uncacheable routes (polls etc.): return a
  `Cache-Control: no-store` or `Cache-Control: private` header from the
  renderer. For dynamic responses that should remain cached, add a data-version
  segment to the cache key in `deploy/src/cache.ts`. Do not sprinkle
  route-specific cache logic elsewhere.
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

## Site data database (D1)

- `benjisponge-spire` is the existing shared site-data D1 database, bound as
  `SITE_DB`. The name is historical; changing the binding did not replace or
  copy the database. From `deploy/`, `spire-schema.sql` is idempotent and
  `fitness-schema.sql` bootstraps an empty fitness archive:
  `npx wrangler d1 execute benjisponge-spire --remote --file=spire-schema.sql`
  `npx wrangler d1 execute benjisponge-spire --remote --file=fitness-schema.sql`
  `fitness-schema.sql` is for an empty fitness archive. To replace an older
  fitness archive, reset and resync its six fitness tables as documented in
  `docs/fitness.md`; never drop the shared D1 database or its Spire tables.

### Spire runs

- API in `src/spire.ts`: `GET /api/spire/runs` and `GET /api/spire/ids` are
  public; `POST /api/spire/runs` needs the `SPIRE_SYNC_TOKEN` Worker secret
  as a Bearer token. The local copy the CLI reads lives at
  `~/.config/benjisponge/spire.token`. Rotate by regenerating the file
  (`openssl rand -hex 32 > ~/.config/benjisponge/spire.token`) then
  `npx wrangler secret put SPIRE_SYNC_TOKEN < ~/.config/benjisponge/spire.token`
- Write path is `just sync-spire` (`src/bin/spire_sync.rs`) on the machine
  with the save files. Idempotent twice over: the CLI diffs against
  `/api/spire/ids`, and the server INSERT OR IGNOREs by run id.
- Every insert bumps `spire_meta.version`. The container renders `/spire`,
  `/`, and `/feed.xml` from `GET /api/spire/runs` (60 s in-process cache),
  and those three paths embed the version in their edge-cache key
  (`DATA_VERSIONED` in `cache.ts`) — so a sync invalidates exactly those
  pages on the next request, with no deploy and no purge call.

### Fitness archive

Cross-layer map, source invariants, taxonomy workflow, and manual-logging
boundary: `docs/fitness.md`.

`just dev [port]` starts the Worker on `127.0.0.1:8791` with a fixed local-only
import token, then starts Topcoat on the requested port. Exiting Topcoat also
stops Wrangler. It does not initialize, migrate, or sync D1.
`just reset-fitness-local [csv]`, run in another terminal while dev is active,
drops only the six local fitness tables, recreates `fitness-schema.sql`, and
imports the CSV (default `/home/benji/Downloads/WorkoutData.csv`). Local Spire
tables in the shared persistent D1 database remain untouched.

`fitness-schema.sql` creates six normalized STRICT tables: `workouts`,
`exercises`, `exercise_tags`, `sets`, `set_records`, and `fitness_meta`.
Each workout keeps Strong's original UTC start as `started_at_utc`, its
`America/New_York` wall-clock projection as `started_at_local`, and
`eastern_offset_minutes` (`-240` for EDT or `-300` for EST). Strong's
offset-less `Date` column is UTC; it is never interpreted in the Worker or
developer machine's timezone. Set ordinals remain 1-based within the workout.
Numeric decimals are stored losslessly as scaled integers: load and distance in
thousandths, effort in hundredths. Imported load is assumed to be pounds and
stored with `weight_unit="lbs"`. Effort is stored as RPE: source values below 6
are interpreted as RIR and converted with `RPE = 10 - RIR`.

The UTC/Eastern archive is a fresh fitness schema, not an in-place upgrade.
For the existing remote archive, use the fitness-only reset-and-resync procedure
in `docs/fitness.md`. It drops only `set_records`, `sets`, `exercise_tags`,
`exercises`, `workouts`, and `fitness_meta`, then recreates them from
`fitness-schema.sql`; shared Spire tables remain intact.

Public reads are `Cache-Control: no-store` and include
`Access-Control-Allow-Origin: *`:

- `GET /api/fitness/sets` returns a workout-grouped page of matching sets:
  `{version,page,per_page,total_sets,total_workouts,workouts}`. Each workout is
  `{id,path,title,raw_title,started_at_local,ended_at_local,eastern_offset_minutes,end_eastern_offset_minutes,duration_seconds,duration_suspicious,notes,description,sets}`.
  `id` stays an opaque UTC-derived stable identifier; `path` is the canonical
  public path segment. Reader responses do not expose a `started_at_utc`
  field; all user-facing times are Eastern.
  each set is
  `{id,ordinal,exercise_name,raw_exercise_name,exercise_note,superset_id,weight_milli,weight_unit,reps,effort_hundredths,distance_milli,set_time_seconds,set_type,records}`;
  each record is `{level,kind}`. Pagination is by whole workout, so a workout's
  matching sets are never split across pages. `total_sets` and
  `total_workouts` cover the entire filtered result, not just the page.
- `GET /api/fitness/calendar` accepts no query parameters and returns
  `{version,days:[{date,volume_points}]}` for every `America/New_York` date
  with at least one set, in ascending date order. `volume_points` follows the
  site set-log score exactly: warm-up = 0, failure = 6, RPE 10/9/8 = 5/4/3,
  and any other or missing effort = 2.
- `GET /api/fitness/workouts/latest` accepts no query parameters and returns
  `{version,workout,newer_workout_path,older_workout_path}` for the newest
  workout by source instant. `workout` has the same shape as a
  `sets`-response workout and is `null` for an empty archive; both neighbor
  paths are then `null` too.
- `GET /api/fitness/workouts/by-path/{path}` accepts one canonical public path
  segment, such as `2026-07-11T20-33-27-04-00`, and returns the same detail
  envelope or 404. The timestamp and offset are the `America/New_York`
  projection, so the offset distinguishes the repeated hour when DST ends.
- `GET /api/fitness/facets` accepts no query parameters and returns
  `{version,summary:{sets,workouts,min_date,max_date},exercises,tags,set_types}`.
  Exercise, tag, and set-type entries are `{value,count}`; `tags` has
  `movement`, `muscle`, and `equipment` arrays. Counts cover the whole archive.
- `GET /api/fitness/ids` accepts no query parameters and returns
  `{ids:string[]}` containing set IDs. The sync command uses these to resume at
  set granularity.

`GET /api/fitness/sets` accepts only these query parameters:

- Text/facets: `q`; exact `exercise`; repeated `movement`, `muscle`,
  `equipment`, and `set_type`. Repeated choices are ORed within one facet and
  different filters are ANDed. `q` searches workout titles/notes/description,
  exercise names, raw exercise names, and exercise notes.
- Dates: inclusive `from`/`to` (`YYYY-MM-DD`); `weekday` = `sun` through `sat`;
  `time_of_day` = `morning` (05:00-11:59), `afternoon` (12:00-16:59),
  `evening` (17:00-20:59), or `night` (21:00-04:59). All date, weekday, and
  time-of-day filtering uses `America/New_York`, including DST transitions.
- Numbers: `min_load`/`max_load` are pounds converted exactly to
  stored thousandths; `min_reps`/`max_reps` are integers; `max_effort` is a
  decimal converted exactly to stored hundredths.
- Flags: `has_record`, `has_superset`, `has_notes`, and `incomplete` accept
  `true` or `false`; `duration` is `normal` or `suspicious`. A set is
  incomplete when reps, distance, and set duration are all absent.
- Pagination: positive `page`; `per_page` is exactly `10`, `20`, or `40`
  (default `20`).

Unknown, duplicated singular, malformed, out-of-range, or contradictory
filters return 400. Repeated facets are capped at eight values each. Search is
also checked against D1's 50-byte escaped LIKE-pattern limit.

The write path is `POST /api/fitness/import`, protected by the
`FITNESS_SYNC_TOKEN` Worker secret. The body is capped at 1,000,000 bytes, 50
sets, 50 workouts, 75 exercises, 300 tags, and 200 records. Its exact shape is:

```text
{
  workouts: [{
    id, title, raw_title, started_at_utc, duration_seconds,
    duration_suspicious, notes, description, source: "workout-data-csv"
  }],
  exercises: [{
    name, tags: [{kind: "movement"|"muscle"|"equipment", value}]
  }],
  sets: [{
    id, workout_id, ordinal, exercise_name, raw_exercise_name, exercise_note,
    superset_id, weight_milli, weight_unit: "lbs", reps, effort_hundredths, distance_milli,
    set_time_seconds, set_type,
    records: [{level: "gold"|"silver"|"bronze",
               kind: "1rm"|"max-weight"|"volume"|"reps"}]
  }]
}
```

Nullable fields must be explicit JSON `null`. IDs, cross-references, UTC source
dates, ordinals, scaled integers, enum values, unique record kinds, and every
string/array bound are validated before any write. The Worker derives the
Eastern fields; callers never supply them. The response is
`{received,added,skipped,version}`, where the counts refer to sets. Existing set
IDs are skipped; a conflicting workout ordinal is an error rather than being
silently ignored. Tags are replaced authoritatively for each exercise included
in a chunk. The fitness version increments only when sets or taxonomy change.
The CSV path is deliberately append-oriented: an already stored set ID is
immutable. Editing, reordering, or deleting old rows in a later export requires
an explicit replacement operation rather than silently rewriting history.
The normal CLI only posts workouts containing a missing set, so taxonomy-only
changes on a fully imported archive likewise need a deliberate re-import/API
call (or will arrive when that exercise is included with a later missing set).

`/lifting`, `/lifting/log`, and permanent `/lifting/{path}` pages fetch these
public reads from the Topcoat server and return complete HTML. The landing page
reads `calendar` and `workouts/latest`; the full log reads `facets` and `sets`;
a permanent page resolves its canonical Eastern path through
`workouts/by-path/{path}`. Canonical paths include the local timestamp and the
offset, for example `/lifting/2026-07-11T20-33-27-04-00`. Raw importer IDs and
earlier timestamp shapes are not public routes. The small browser enhancement
only debounces the full log's native filter form and navigates to a new GET URL.
These routes return `Cache-Control: no-store`, so the generic edge policy leaves
their HTML unstored and an import is visible on the next request. `just dev`
sets `FITNESS_DATA_ORIGIN` to the local Worker; production uses `SITE_ORIGIN`.

Sync from the machine that has the export:

```sh
just sync-fitness /home/benji/Downloads/WorkoutData.csv --dry-run
just sync-fitness /home/benji/Downloads/WorkoutData.csv
```

The default token file is `~/.config/benjisponge/fitness.token`. From `deploy/`,
install or rotate the matching Worker secret with:

```sh
npx wrangler secret put FITNESS_SYNC_TOKEN < ~/.config/benjisponge/fitness.token
```

## One-time account setup (manual)

Workers Paid plan; benjisponge.com zone on Cloudflare DNS; GitHub secrets
CLOUDFLARE_API_TOKEN + CLOUDFLARE_ACCOUNT_ID; first deploy from local
(`npx wrangler login` then `just deploy`) to provision the DO migration and
custom domains.
