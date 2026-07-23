CREATE TABLE "spire_meta" (
    "k" TEXT NOT NULL,
    "v" BIGINT NOT NULL,
    PRIMARY KEY ("k")
);
CREATE TABLE "spire_run_raws" (
    "id" TEXT NOT NULL,
    "raw" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE TABLE "spire_runs" (
    "id" TEXT NOT NULL,
    "date" TEXT NOT NULL,
    "start_time" BIGINT NOT NULL,
    "character" TEXT NOT NULL,
    "win" BOOLEAN NOT NULL,
    "abandoned" BOOLEAN NOT NULL,
    "ascension" BIGINT NOT NULL,
    "acts" BIGINT NOT NULL,
    "floors" BIGINT NOT NULL,
    "killed_by" TEXT,
    "kill_kind" TEXT,
    "run_time" BIGINT NOT NULL,
    "seed" TEXT NOT NULL,
    "game_mode" TEXT NOT NULL,
    "build_id" TEXT NOT NULL,
    "added_at" BIGINT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE INDEX "index_spire_runs_by_start_time" ON "spire_runs" ("start_time");
