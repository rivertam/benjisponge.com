-- Fitness set archive (see src/fitness.ts for the API over it).
-- Idempotent for an empty fitness archive; apply after a fitness-only reset with:
--   cd deploy && npx wrangler d1 execute benjisponge-spire --remote --file=fitness-schema.sql

PRAGMA foreign_keys = ON;

-- Strong's source `Date` is UTC. Keep that canonical instant for stable IDs
-- and chronological ordering, then materialize its America/New_York civil
-- clock for every reader-facing date, filter, calendar cell, and URL.
CREATE TABLE IF NOT EXISTS workouts (
  id                  TEXT PRIMARY KEY,
  title               TEXT NOT NULL,
  raw_title           TEXT NOT NULL,
  started_at_utc      TEXT NOT NULL,
  started_at_local    TEXT NOT NULL,
  eastern_offset_minutes INTEGER NOT NULL CHECK (eastern_offset_minutes IN (-300, -240)),
  duration_seconds    INTEGER NOT NULL CHECK (duration_seconds BETWEEN 0 AND 604800),
  duration_suspicious INTEGER NOT NULL CHECK (duration_suspicious IN (0, 1)),
  notes               TEXT,
  description         TEXT,
  source              TEXT NOT NULL CHECK (source IN ('workout-data-csv', 'manual')),
  imported_at         INTEGER NOT NULL,
  CHECK (length(id) BETWEEN 1 AND 128),
  CHECK (length(title) BETWEEN 1 AND 240),
  CHECK (length(raw_title) BETWEEN 1 AND 240),
  CHECK (started_at_utc GLOB '????-??-?? ??:??:??'),
  CHECK (started_at_local GLOB '????-??-?? ??:??:??'),
  CHECK (notes IS NULL OR length(notes) <= 10000),
  CHECK (description IS NULL OR length(description) <= 10000)
) STRICT;

CREATE TABLE IF NOT EXISTS exercises (
  name TEXT PRIMARY KEY,
  CHECK (length(name) BETWEEN 1 AND 240)
) STRICT;

-- A movement can carry several tags in each facet. For example, Bulgarian
-- split squats can be movement=squat-type, muscle=quads, and equipment=dumbbell.
CREATE TABLE IF NOT EXISTS exercise_tags (
  exercise_name TEXT NOT NULL REFERENCES exercises(name) ON DELETE CASCADE ON UPDATE CASCADE,
  kind        TEXT NOT NULL CHECK (kind IN ('movement', 'muscle', 'equipment')),
  value       TEXT NOT NULL CHECK (
    length(value) BETWEEN 1 AND 64
    AND value NOT GLOB '*[^a-z0-9-]*'
  ),
  PRIMARY KEY (exercise_name, kind, value)
) STRICT, WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS sets (
  id                 TEXT PRIMARY KEY,
  workout_id         TEXT NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
  exercise_name      TEXT NOT NULL REFERENCES exercises(name) ON UPDATE CASCADE,
  raw_exercise_name  TEXT NOT NULL,
  ordinal            INTEGER NOT NULL CHECK (ordinal BETWEEN 1 AND 10000),
  exercise_note      TEXT,
  superset_id        INTEGER,
  weight_milli       INTEGER CHECK (weight_milli BETWEEN 0 AND 1000000000),
  weight_unit        TEXT NOT NULL DEFAULT 'lbs' CHECK (weight_unit = 'lbs'),
  reps               INTEGER CHECK (reps BETWEEN 0 AND 1000000),
  effort_hundredths  INTEGER CHECK (effort_hundredths BETWEEN 0 AND 100000),
  distance_milli     INTEGER CHECK (distance_milli BETWEEN 0 AND 1000000000),
  set_time_seconds   INTEGER CHECK (set_time_seconds BETWEEN 0 AND 604800),
  set_type           TEXT NOT NULL,
  incomplete         INTEGER NOT NULL CHECK (incomplete IN (0, 1)),
  CHECK (length(id) BETWEEN 1 AND 128),
  CHECK (exercise_note IS NULL OR length(exercise_note) <= 2000),
  CHECK (length(raw_exercise_name) BETWEEN 1 AND 240),
  CHECK (superset_id IS NULL OR superset_id BETWEEN 0 AND 1000000),
  CHECK (set_type IN (
    'WARMUP_SET', 'NORMAL_SET', 'FAILURE_SET', 'PARTIAL_REPS_SET',
    'DROP_SET', 'NEGATIVE_REPS_SET'
  )),
  UNIQUE (workout_id, ordinal)
) STRICT;

CREATE TABLE IF NOT EXISTS set_records (
  set_id  TEXT NOT NULL REFERENCES sets(id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL CHECK (ordinal BETWEEN 1 AND 100),
  level   TEXT NOT NULL CHECK (level IN ('gold', 'silver', 'bronze')),
  kind    TEXT NOT NULL CHECK (kind IN ('1rm', 'max-weight', 'volume', 'reps')),
  PRIMARY KEY (set_id, ordinal),
  UNIQUE (set_id, kind)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS workouts_started_at_utc
  ON workouts (started_at_utc DESC, id DESC);
CREATE INDEX IF NOT EXISTS workouts_started_at_local
  ON workouts (started_at_local DESC, id DESC);
CREATE INDEX IF NOT EXISTS workouts_public_path
  ON workouts (started_at_local, eastern_offset_minutes);
CREATE INDEX IF NOT EXISTS sets_exercise
  ON sets (exercise_name);
CREATE INDEX IF NOT EXISTS sets_type
  ON sets (set_type);
CREATE INDEX IF NOT EXISTS sets_weight
  ON sets (weight_milli) WHERE weight_milli IS NOT NULL;
CREATE INDEX IF NOT EXISTS sets_reps
  ON sets (reps) WHERE reps IS NOT NULL;
CREATE INDEX IF NOT EXISTS sets_effort
  ON sets (effort_hundredths) WHERE effort_hundredths IS NOT NULL;
CREATE INDEX IF NOT EXISTS exercise_tags_facet
  ON exercise_tags (kind, value, exercise_name);

-- POST /api/fitness/import bumps this once for each chunk that adds sets.
CREATE TABLE IF NOT EXISTS fitness_meta (
  k TEXT PRIMARY KEY,
  v INTEGER NOT NULL CHECK (v >= 0)
) STRICT;
INSERT OR IGNORE INTO fitness_meta (k, v) VALUES ('version', 0);
