-- The Slay the Spire 2 run archive (see src/spire.ts for the API over it).
-- Idempotent; apply with:
--   cd deploy && npx wrangler d1 execute benjisponge-spire --remote --file=spire-schema.sql

CREATE TABLE IF NOT EXISTS spire_runs (
  id         TEXT PRIMARY KEY, -- run file stem == start_time as a string
  date       TEXT NOT NULL,    -- YYYY-MM-DD, US Eastern, stamped at sync
  start_time INTEGER NOT NULL, -- epoch seconds; the sort key
  character  TEXT NOT NULL,    -- prettified; co-op joins with " + "
  win        INTEGER NOT NULL, -- 0/1
  abandoned  INTEGER NOT NULL, -- 0/1
  ascension  INTEGER NOT NULL,
  acts       INTEGER NOT NULL, -- acts reached
  floors     INTEGER NOT NULL, -- map nodes visited across all acts
  killed_by  TEXT,             -- prettified; NULL on wins/abandons
  kill_kind  TEXT,             -- boss | elite | monster | event
  run_time   INTEGER NOT NULL, -- seconds of play
  seed       TEXT NOT NULL,
  game_mode  TEXT NOT NULL,    -- standard | daily
  build_id   TEXT NOT NULL,    -- game version that produced the run
  raw        TEXT NOT NULL,    -- the entire original .run file
  added_at   INTEGER NOT NULL  -- epoch seconds of the sync that inserted it
);

CREATE INDEX IF NOT EXISTS spire_runs_start_time ON spire_runs (start_time DESC);

-- Single-row version counter; POST bumps it, and rendered pages embed it in
-- their edge-cache key so a sync invalidates them without a deploy.
CREATE TABLE IF NOT EXISTS spire_meta (k TEXT PRIMARY KEY, v INTEGER NOT NULL);
INSERT OR IGNORE INTO spire_meta (k, v) VALUES ('version', 0);
