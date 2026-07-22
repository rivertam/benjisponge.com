# Fitness archive

Read this before changing `/lifting`, workout import, fitness API/schema, tags,
or local fitness startup. Exact API/filter/import contracts live in
`docs/cloudflare-deploy.md#fitness-archive`.

## Data flow

- Page/filter markup: `src/app/interests/lifting.rs`; browser query/rendering:
  `src/app/interests/lifting/fitness.js`; styles: `styles/lifting.css`.
- Public reads and authenticated imports: `deploy/src/fitness.ts`; D1 schema:
  `deploy/fitness-schema.sql`.
- CSV parsing, stable IDs, taxonomy, chunking: `src/bin/fitness_sync.rs`.
- `just dev [port]` delegates to `scripts/dev.sh`: schema -> Worker on 8791 ->
  idempotent CSV sync -> Topcoat. Wrangler stops when Topcoat exits.

## Source invariants

- CSV stays outside git. Audited baseline: 5,561 sets, 360 workouts, 221
  exercises, 2023-09-27 through 2026-07-21; 548 squat-type sets in 97 workouts.
- Export has no per-set timestamp or timezone. Every set uses its workout's
  local-naive start/end time; never guess UTC.
- Export omits load/distance units and labels effort `RIR/RPE`; never infer more.
- Preserve apparent duplicate rows. Set identity is workout local start plus
  whole-workout ordinal; deduping or reordering changes IDs.
- Duration `0` or at least four hours is suspicious, not invalid. Preserve it.
- D1 stores load/distance in thousandths and effort in hundredths. Keep integer
  scaling and explicit JSON nulls across importer, API, and UI.

## Local development

- Default CSV: `/home/benji/Downloads/WorkoutData.csv`; override with
  `WORKOUT_DATA_CSV=/path/export.csv just dev [port]`.
- Local D1 persists under ignored `deploy/.wrangler/state`. Stop `just dev`
  before deleting that directory for a clean rebuild.
- Port 8791 is reserved for Wrangler. Local `/lifting` falls back to that API.

## Changing taxonomy or filters

- Taxonomy originates in `exercise_tags()` and `SQUAT_TYPE_EXERCISES`; update
  importer tests with every classification rule.
- Keep taxonomy values aligned with filter lists in `lifting.rs` and labels /
  advanced-filter detection in `fitness.js`.
- Normal sync is append-only. It does not resend fully imported workouts for a
  taxonomy-only change. No retag command exists: write an explicit API/D1
  migration instead of rerunning normal sync. Reset local D1 when validating.
- Do not substring-match movement names without boundary tests: `throw` contains
  `row`; wrist/Jefferson curls are not biceps curls.

## Production and future logging

- Rollout order: apply `fitness-schema.sql` remotely, install
  `FITNESS_SYNC_TOKEN`, deploy committed HEAD, then run `just sync-fitness`.
- `CREATE TABLE IF NOT EXISTS` bootstraps safely but does not evolve existing
  tables. Column or constraint changes require an explicit migration.
- Never treat local D1 contents as proof production is migrated or seeded.
- Manual logging is not implemented. Schema reserves `source='manual'`, but the
  import API accepts only `workout-data-csv`; there is no CRUD UI, mutation ID
  policy, or owner auth yet. Decide those explicitly before adding write UI.

## Validation

```sh
just check
just build
node --check src/app/interests/lifting/fitness.js
bash -n scripts/dev.sh
cd deploy && npx wrangler types --check && npx tsc --noEmit
```

No committed Worker integration suite exists yet. For Worker/API changes, also
exercise a fresh local import, filtered reads, idempotent second sync, and
shutdown cleanup through `just dev`.
