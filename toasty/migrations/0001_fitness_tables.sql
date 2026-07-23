CREATE TABLE "exercise_tags" (
    "exercise_name" TEXT NOT NULL,
    "kind" TEXT NOT NULL,
    "value" TEXT NOT NULL,
    PRIMARY KEY ("exercise_name", "kind", "value")
);
CREATE TABLE "sets" (
    "id" TEXT NOT NULL,
    "workout_id" TEXT NOT NULL,
    "exercise_name" TEXT NOT NULL,
    "raw_exercise_name" TEXT NOT NULL,
    "ordinal" BIGINT NOT NULL,
    "exercise_note" TEXT,
    "superset_id" BIGINT,
    "weight_milli" BIGINT,
    "weight_unit" TEXT NOT NULL,
    "reps" BIGINT,
    "effort_hundredths" BIGINT,
    "distance_milli" BIGINT,
    "set_time_seconds" BIGINT,
    "set_type" TEXT NOT NULL,
    "incomplete" BOOLEAN NOT NULL,
    PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "index_sets_by_workout_id_and_ordinal" ON "sets" ("workout_id", "ordinal");
CREATE INDEX "index_sets_by_workout_id" ON "sets" ("workout_id");
CREATE TABLE "fitness_meta" (
    "k" TEXT NOT NULL,
    "v" BIGINT NOT NULL,
    PRIMARY KEY ("k")
);
CREATE TABLE "exercises" (
    "name" TEXT NOT NULL,
    PRIMARY KEY ("name")
);
CREATE TABLE "workouts" (
    "id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "raw_title" TEXT NOT NULL,
    "started_at_utc" TEXT NOT NULL,
    "started_at_local" TEXT NOT NULL,
    "eastern_offset_minutes" BIGINT NOT NULL,
    "duration_seconds" BIGINT NOT NULL,
    "duration_suspicious" BOOLEAN NOT NULL,
    "notes" TEXT,
    "description" TEXT,
    "source" TEXT NOT NULL,
    "imported_at" BIGINT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE INDEX "index_workouts_by_started_at_utc" ON "workouts" ("started_at_utc");
-- Hand-added (toasty generates DDL only): the import endpoint's version
-- counter increments from this row, mirroring fitness-schema.sql's seed.
INSERT INTO "fitness_meta" ("k", "v") VALUES ('version', 0);
