# Fitness archive

Read this before changing `/lifting`, workout import, fitness API/schema, tags,
or local fitness startup. Exact API/filter/import contracts live in
`docs/cloudflare-deploy.md#fitness-archive`.

## Data flow

- Page, filter/query handling, API reader, and HTML rendering:
  `src/app/interests/lifting/`; styles: `styles/lifting.css`. `/lifting` is the
  no-JavaScript landing view (daily volume heatmap plus the newest lift),
  `/lifting/log` is the filterable full archive, and
  `/lifting/YYYY-MM-DDTHH-MM-SS-04-00` (or `-05-00`) is a complete permanent
  workout page. Its timestamp is the `America/New_York` projection of the
  source instant; the explicit Eastern offset keeps same-date workouts and the
  repeated fall DST hour distinct without exposing importer IDs.
  `auto-filter.js` only debounces full-log form changes and navigates to the
  canonical GET URL; the Apply button remains the no-JavaScript path.
- Public reads and authenticated imports: `deploy/src/fitness.ts`; fresh D1
  schema: `deploy/fitness-schema.sql`.
- CSV parsing, stable IDs, taxonomy, chunking: `src/bin/fitness_sync.rs`.
- `just dev [port]` delegates to `scripts/dev.sh`: fresh schema -> Worker on
  8791 -> idempotent CSV sync -> Topcoat. It does not migrate or reset an
  existing archive. Wrangler stops when Topcoat exits.

## Source invariants

- CSV stays outside git. Audited baseline: 5,561 sets, 360 workouts, 221
  exercises, 2023-09-27 through 2026-07-21; 548 squat-type sets in 97 workouts.
- Strong's offset-less `Date` field is always UTC. Parse it as UTC, never as
  the machine's timezone or a local-naive timestamp.
- Keep the source instant as `started_at_utc`, then derive
  `started_at_local` and `eastern_offset_minutes` with the IANA
  `America/New_York` rules. This means Eastern time (EST *and* EDT), not a
  fixed EST offset. All public dates, calendar buckets, date/weekday/
  time-of-day filters, labels, and permanent lift URLs use that Eastern
  projection.
- Stable workout and set IDs remain derived from the raw UTC start timestamp
  (and the whole-workout ordinal for sets). Timezone conversion must never
  change identity, deduplication, or import ordering.
- Export omits load/distance units and labels effort `RIR/RPE`; never infer more.
- Preserve apparent duplicate rows. Set identity is workout UTC start plus
  whole-workout ordinal; deduping or reordering changes IDs.
- Duration `0` or at least four hours is suspicious, not invalid. Preserve it.
- D1 stores load/distance in thousandths and effort in hundredths. Keep integer
  scaling and explicit JSON nulls across importer, API, and UI.

## Local development

- Default CSV: `/home/benji/Downloads/WorkoutData.csv`; override with
  `WORKOUT_DATA_CSV=/path/export.csv just dev [port]`.
- Local D1 persists under ignored `deploy/.wrangler/state`. Stop `just dev`
  before deleting that directory for a clean rebuild. A local archive created
  before `started_at_utc` existed must be reset once before using the simplified
  startup flow:

  ```sh
  rm -rf deploy/.wrangler/state
  ```

  This is local development state only. It also removes any local Spire fixtures
  in that shared D1; it never affects production.
- Port 8791 is reserved for Wrangler. `just dev` points `/lifting`'s server-side
  API reader at that local Worker.

## Changing taxonomy or filters

- Taxonomy originates in `exercise_tags()` and `SQUAT_TYPE_EXERCISES`; update
  importer tests with every classification rule.
- Keep taxonomy values aligned with the filter lists, labels, and
  advanced-filter detection in `src/app/interests/lifting/filters.rs`.
- Normal sync is append-only. It does not resend fully imported workouts for a
  taxonomy-only change. No retag command exists: write an explicit API/D1
  replacement workflow instead of rerunning normal sync. Reset local D1 when
  validating.
- Do not substring-match movement names without boundary tests: `throw` contains
  `row`; wrist/Jefferson curls are not biceps curls.

## Production and future logging

- For a new archive, rollout order is: apply `fitness-schema.sql` remotely,
  install `FITNESS_SYNC_TOKEN`, deploy committed HEAD, then run
  `just sync-fitness`.
- The fitness archive intentionally uses reset-and-resync rather than an
  in-place schema upgrade.
  To replace the existing production fitness data with the UTC/Eastern schema,
  drop exactly the six fitness tables, recreate them, deploy committed HEAD,
  then run `just sync-fitness` from the machine with the CSV:

  ```sh
  cd deploy
  npx wrangler d1 execute benjisponge-spire --remote --command \
    "DROP TABLE IF EXISTS set_records;
     DROP TABLE IF EXISTS sets;
     DROP TABLE IF EXISTS exercise_tags;
     DROP TABLE IF EXISTS exercises;
     DROP TABLE IF EXISTS workouts;
     DROP TABLE IF EXISTS fitness_meta;"
  npx wrangler d1 execute benjisponge-spire --remote --file=fitness-schema.sql
  ```

  This is destructive for fitness history until the CSV is resynced. `SITE_DB`
  is shared with Slay the Spire data: do not drop the database itself and do not
  drop or alter any Spire tables. The commands above touch fitness tables only.
- Never treat local D1 contents as proof that production has been reset or
  seeded.
- Manual logging is not implemented. Schema reserves `source='manual'`, but the
  import API accepts only `workout-data-csv`; there is no CRUD UI, mutation ID
  policy, or owner auth yet. Decide those explicitly before adding write UI.

## Validation

```sh
just check
just build
node --check src/app/interests/lifting/auto-filter.js
bash -n scripts/dev.sh
cd deploy && npx wrangler types --check && npx tsc --noEmit
```

No committed Worker integration suite exists yet. For Worker/API changes, also
exercise a fresh local import, filtered reads, idempotent second sync, and
shutdown cleanup through `just dev`.
