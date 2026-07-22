// /api/fitness/* -- the public fitness archive and private bounded import path.
// Source timestamps are local wall-clock strings; no timezone is inferred.
// Cross-layer invariants and taxonomy workflow: ../../docs/fitness.md.

import { bearerAuthorized } from "./auth";

type FitnessEnv = Env & { FITNESS_SYNC_TOKEN?: string };
type TagKind = "movement" | "muscle" | "equipment";
type TimeOfDay = "morning" | "afternoon" | "evening" | "night";

type IncomingWorkout = {
  id: string;
  title: string;
  raw_title: string;
  started_at_local: string;
  duration_seconds: number;
  duration_suspicious: boolean;
  notes: string | null;
  description: string | null;
  source: "workout-data-csv";
};

type IncomingTag = { kind: TagKind; value: string };

type IncomingExercise = {
  name: string;
  tags: IncomingTag[];
};

type IncomingRecord = {
  level: "gold" | "silver" | "bronze";
  kind: "1rm" | "max-weight" | "volume" | "reps";
};

type IncomingSet = {
  id: string;
  workout_id: string;
  ordinal: number;
  exercise_name: string;
  raw_exercise_name: string;
  exercise_note: string | null;
  superset_id: number | null;
  weight_milli: number | null;
  reps: number | null;
  effort_hundredths: number | null;
  distance_milli: number | null;
  set_time_seconds: number | null;
  set_type: string;
  records: IncomingRecord[];
};

type ImportPayload = {
  workouts: IncomingWorkout[];
  exercises: IncomingExercise[];
  sets: IncomingSet[];
};

type FilterSql = {
  where: string;
  params: Array<string | number>;
  page: number;
  perPage: 10 | 20 | 40;
};

type CountRow = { total_sets: number; total_workouts: number };
type IdRow = { id: string };

type SetRow = {
  workout_id: string;
  title: string;
  raw_title: string;
  started_at_local: string;
  duration_seconds: number;
  duration_suspicious: number;
  workout_notes: string | null;
  workout_description: string | null;
  set_id: string;
  ordinal: number;
  exercise_name: string;
  raw_exercise_name: string;
  exercise_note: string | null;
  superset_id: number | null;
  weight_milli: number | null;
  reps: number | null;
  effort_hundredths: number | null;
  distance_milli: number | null;
  set_time_seconds: number | null;
  set_type: string;
};

type RecordRow = {
  set_id: string;
  ordinal: number;
  level: string;
  kind: string;
};

type StoredTagRow = { exercise_name: string; kind: string; value: string };

type ApiRecord = { level: string; kind: string };
type ApiSet = {
  id: string;
  ordinal: number;
  exercise_name: string;
  raw_exercise_name: string;
  exercise_note: string | null;
  superset_id: number | null;
  weight_milli: number | null;
  reps: number | null;
  effort_hundredths: number | null;
  distance_milli: number | null;
  set_time_seconds: number | null;
  set_type: string;
  records: ApiRecord[];
};

type ApiWorkout = {
  id: string;
  title: string;
  raw_title: string;
  started_at_local: string;
  duration_seconds: number;
  duration_suspicious: boolean;
  notes: string | null;
  description: string | null;
  sets: ApiSet[];
};

const BODY_LIMIT_BYTES = 1_000_000;
const MAX_IMPORT_SETS = 50;
const MAX_IMPORT_WORKOUTS = 50;
const MAX_IMPORT_EXERCISES = 75;
const MAX_TOTAL_TAGS = 300;
const MAX_TOTAL_RECORDS = 200;
const DUPLICATE_PARAM = Symbol("duplicate query parameter");
const ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$/;
const TAG_PATTERN = /^[a-z0-9][a-z0-9-]{0,63}$/;
const SET_TYPE_PATTERN = /^(?:WARMUP_SET|NORMAL_SET|FAILURE_SET|PARTIAL_REPS_SET|DROP_SET|NEGATIVE_REPS_SET)$/;
const PUBLIC_HEADERS = { "Access-Control-Allow-Origin": "*" };
const TAG_KINDS: readonly TagKind[] = ["movement", "muscle", "equipment"];
const ALLOWED_FILTERS = new Set([
  "q",
  "movement",
  "muscle",
  "equipment",
  "set_type",
  "exercise",
  "from",
  "to",
  "time_of_day",
  "weekday",
  "min_load",
  "max_load",
  "min_reps",
  "max_reps",
  "max_effort",
  "has_record",
  "has_superset",
  "has_notes",
  "incomplete",
  "duration",
  "page",
  "per_page",
]);

export async function handleFitness(
  request: Request,
  env: FitnessEnv,
  url: URL,
): Promise<Response> {
  const isPublicRead = request.method === "GET";
  try {
    if (request.method === "GET" && url.pathname === "/api/fitness/sets") {
      return await listSets(env, url);
    }
    if (request.method === "GET" && url.pathname === "/api/fitness/facets") {
      return await listFacets(env, url);
    }
    if (request.method === "GET" && url.pathname === "/api/fitness/ids") {
      return await listIds(env, url);
    }
    if (request.method === "POST" && url.pathname === "/api/fitness/import") {
      return await importChunk(request, env);
    }
  } catch (error) {
    console.error(
      JSON.stringify({
        message: "fitness api failed",
        path: url.pathname,
        error: error instanceof Error ? error.message : String(error),
      }),
    );
    return json({ error: "internal error" }, 500, isPublicRead);
  }
  return json({ error: "not found" }, 404, isPublicRead);
}

export async function fitnessDataVersion(env: Env): Promise<number> {
  const version = await env.SITE_DB.prepare(
    "SELECT v FROM fitness_meta WHERE k = 'version'",
  ).first<number>("v");
  return version ?? 0;
}

async function listSets(env: FitnessEnv, url: URL): Promise<Response> {
  const parsed = parseFilters(url.searchParams);
  if (typeof parsed === "string") return badFilter(parsed);

  const countStatement = env.SITE_DB.prepare(
    `SELECT COUNT(*) AS total_sets,
            COUNT(DISTINCT s.workout_id) AS total_workouts
       FROM sets s
       JOIN workouts w ON w.id = s.workout_id
       JOIN exercises e ON e.name = s.exercise_name
      WHERE ${parsed.where}`,
  ).bind(...parsed.params);

  const offset = (parsed.page - 1) * parsed.perPage;
  const workoutStatement = env.SITE_DB.prepare(
    `SELECT w.id
       FROM workouts w
      WHERE EXISTS (
        SELECT 1
          FROM sets s
          JOIN exercises e ON e.name = s.exercise_name
         WHERE s.workout_id = w.id AND ${parsed.where}
      )
      ORDER BY w.started_at_local DESC, w.id DESC
      LIMIT ? OFFSET ?`,
  ).bind(...parsed.params, parsed.perPage, offset);

  const [countResult, workoutResult] = await Promise.all([
    countStatement.all<CountRow>(),
    workoutStatement.all<IdRow>(),
  ]);
  const count = countResult.results[0];
  const workoutIds = workoutResult.results.map((row) => row.id);
  const version = await fitnessDataVersion(env);

  if (workoutIds.length === 0) {
    return json(
      {
        version,
        page: parsed.page,
        per_page: parsed.perPage,
        total_sets: count?.total_sets ?? 0,
        total_workouts: count?.total_workouts ?? 0,
        workouts: [],
      },
      200,
      true,
    );
  }

  const workoutPlaceholders = placeholders(workoutIds.length);
  const rowParams: Array<string | number> = [...workoutIds, ...parsed.params];
  const rowsStatement = env.SITE_DB.prepare(
    `SELECT w.id AS workout_id, w.title, w.raw_title, w.started_at_local,
            w.duration_seconds, w.duration_suspicious,
            w.notes AS workout_notes, w.description AS workout_description,
            s.id AS set_id, s.ordinal, e.name AS exercise_name,
            s.raw_exercise_name, s.exercise_note, s.superset_id,
            s.weight_milli, s.reps, s.effort_hundredths, s.distance_milli,
            s.set_time_seconds, s.set_type
       FROM sets s
       JOIN workouts w ON w.id = s.workout_id
       JOIN exercises e ON e.name = s.exercise_name
      WHERE s.workout_id IN (${workoutPlaceholders})
        AND ${parsed.where}
      ORDER BY w.started_at_local DESC, w.id DESC, s.ordinal ASC`,
  ).bind(...rowParams);
  const recordsStatement = env.SITE_DB.prepare(
    `SELECT sr.set_id, sr.ordinal, sr.level, sr.kind
       FROM set_records sr
       JOIN sets s ON s.id = sr.set_id
       JOIN workouts w ON w.id = s.workout_id
       JOIN exercises e ON e.name = s.exercise_name
      WHERE s.workout_id IN (${workoutPlaceholders})
        AND ${parsed.where}
      ORDER BY sr.set_id, sr.ordinal`,
  ).bind(...rowParams);
  const [rowsResult, recordsResult] = await Promise.all([
    rowsStatement.all<SetRow>(),
    recordsStatement.all<RecordRow>(),
  ]);

  const recordsBySet = new Map<string, ApiRecord[]>();
  for (const row of recordsResult.results) {
    const records = recordsBySet.get(row.set_id) ?? [];
    records.push({ level: row.level, kind: row.kind });
    recordsBySet.set(row.set_id, records);
  }

  const workouts: ApiWorkout[] = [];
  const workoutsById = new Map<string, ApiWorkout>();
  for (const row of rowsResult.results) {
    let workout = workoutsById.get(row.workout_id);
    if (!workout) {
      workout = {
        id: row.workout_id,
        title: row.title,
        raw_title: row.raw_title,
        started_at_local: row.started_at_local,
        duration_seconds: row.duration_seconds,
        duration_suspicious: row.duration_suspicious === 1,
        notes: row.workout_notes,
        description: row.workout_description,
        sets: [],
      };
      workoutsById.set(row.workout_id, workout);
      workouts.push(workout);
    }
    workout.sets.push({
      id: row.set_id,
      ordinal: row.ordinal,
      exercise_name: row.exercise_name,
      raw_exercise_name: row.raw_exercise_name,
      exercise_note: row.exercise_note,
      superset_id: row.superset_id,
      weight_milli: row.weight_milli,
      reps: row.reps,
      effort_hundredths: row.effort_hundredths,
      distance_milli: row.distance_milli,
      set_time_seconds: row.set_time_seconds,
      set_type: row.set_type,
      records: recordsBySet.get(row.set_id) ?? [],
    });
  }

  return json(
    {
      version,
      page: parsed.page,
      per_page: parsed.perPage,
      total_sets: count?.total_sets ?? 0,
      total_workouts: count?.total_workouts ?? 0,
      workouts,
    },
    200,
    true,
  );
}

type SummaryRow = {
  sets: number;
  workouts: number;
  min_date: string | null;
  max_date: string | null;
};
type FacetRow = { value: string; count: number };
type TagFacetRow = FacetRow & { kind: string };

async function listFacets(env: FitnessEnv, url: URL): Promise<Response> {
  if (url.search !== "") return badFilter("facets does not accept filters");

  const [summaryResult, exerciseResult, tagResult, setTypeResult] = await Promise.all([
      env.SITE_DB.prepare(
        `SELECT COUNT(s.id) AS sets,
                COUNT(DISTINCT s.workout_id) AS workouts,
                MIN(substr(w.started_at_local, 1, 10)) AS min_date,
                MAX(substr(w.started_at_local, 1, 10)) AS max_date
           FROM sets s
           JOIN workouts w ON w.id = s.workout_id`,
      ).all<SummaryRow>(),
      env.SITE_DB.prepare(
        `SELECT e.name AS value, COUNT(*) AS count
           FROM sets s
           JOIN exercises e ON e.name = s.exercise_name
          GROUP BY e.name
          ORDER BY count DESC, value COLLATE NOCASE`,
      ).all<FacetRow>(),
      env.SITE_DB.prepare(
        `SELECT et.kind, et.value, COUNT(*) AS count
           FROM exercise_tags et
           JOIN sets s ON s.exercise_name = et.exercise_name
          GROUP BY et.kind, et.value
          ORDER BY et.kind, count DESC, et.value`,
      ).all<TagFacetRow>(),
      env.SITE_DB.prepare(
        `SELECT s.set_type AS value, COUNT(*) AS count
           FROM sets s
          GROUP BY s.set_type
          ORDER BY count DESC, value`,
      ).all<FacetRow>(),
    ]);

  const summary = summaryResult.results[0] ?? {
    sets: 0,
    workouts: 0,
    min_date: null,
    max_date: null,
  };
  const tags: Record<TagKind, FacetRow[]> = {
    movement: [],
    muscle: [],
    equipment: [],
  };
  for (const row of tagResult.results) {
    if (isTagKind(row.kind)) {
      tags[row.kind].push({ value: row.value, count: row.count });
    }
  }
  return json(
    {
      version: await fitnessDataVersion(env),
      summary,
      exercises: exerciseResult.results,
      tags,
      set_types: setTypeResult.results,
    },
    200,
    true,
  );
}

async function listIds(env: FitnessEnv, url: URL): Promise<Response> {
  if (url.search !== "") return badFilter("ids does not accept filters");
  const { results } = await env.SITE_DB.prepare(
    "SELECT id FROM sets ORDER BY id",
  ).all<IdRow>();
  return json({ ids: results.map((row) => row.id) }, 200, true);
}

async function importChunk(
  request: Request,
  env: FitnessEnv,
): Promise<Response> {
  if (!(await bearerAuthorized(request, env.FITNESS_SYNC_TOKEN))) {
    return json({ error: "unauthorized" }, 401);
  }
  const mediaType = request.headers.get("Content-Type")?.split(";", 1)[0]?.trim().toLowerCase();
  if (mediaType !== "application/json") {
    return json({ error: "Content-Type must be application/json" }, 415);
  }

  const decoded = await readJson(request);
  if (typeof decoded === "string") {
    const status = decoded === "body exceeds 1000000 bytes" ? 413 : 400;
    return json({ error: decoded }, status);
  }
  const parsed = parseImportPayload(decoded);
  if (typeof parsed === "string") return json({ error: parsed }, 400);

  const setIds = parsed.sets.map((set) => set.id);
  const existingResult = await env.SITE_DB.prepare(
    `SELECT id FROM sets WHERE id IN (${placeholders(setIds.length)})`,
  )
    .bind(...setIds)
    .all<IdRow>();
  const existing = new Set(existingResult.results.map((row) => row.id));
  const candidates = parsed.sets.filter((set) => !existing.has(set.id));

  const exerciseNames = parsed.exercises.map((exercise) => exercise.name);
  const storedTagResult = await env.SITE_DB.prepare(
    `SELECT exercise_name, kind, value
       FROM exercise_tags
      WHERE exercise_name IN (${placeholders(exerciseNames.length)})
      ORDER BY exercise_name, kind, value`,
  )
    .bind(...exerciseNames)
    .all<StoredTagRow>();
  const storedTags = new Map<string, IncomingTag[]>();
  for (const row of storedTagResult.results) {
    if (!isTagKind(row.kind)) continue;
    const tags = storedTags.get(row.exercise_name) ?? [];
    tags.push({ kind: row.kind, value: row.value });
    storedTags.set(row.exercise_name, tags);
  }
  const changedExercises = new Set(
    parsed.exercises
      .filter(
        (exercise) =>
          tagSignature(exercise.tags) !== tagSignature(storedTags.get(exercise.name) ?? []),
      )
      .map((exercise) => exercise.name),
  );

  if (candidates.length === 0 && changedExercises.size === 0) {
    return json({
      received: parsed.sets.length,
      added: 0,
      skipped: parsed.sets.length,
      version: await fitnessDataVersion(env),
    });
  }

  const statements: D1PreparedStatement[] = [];
  const workoutInsert = env.SITE_DB.prepare(
    `INSERT OR IGNORE INTO workouts
       (id, title, raw_title, started_at_local, duration_seconds,
        duration_suspicious, notes, description, source, imported_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch())`,
  );
  for (const workout of parsed.workouts) {
    statements.push(
      workoutInsert.bind(
        workout.id,
        workout.title,
        workout.raw_title,
        workout.started_at_local,
        workout.duration_seconds,
        workout.duration_suspicious ? 1 : 0,
        workout.notes,
        workout.description,
        workout.source,
      ),
    );
  }

  const exerciseInsert = env.SITE_DB.prepare(
    `INSERT OR IGNORE INTO exercises (name) VALUES (?)`,
  );
  const tagInsert = env.SITE_DB.prepare(
    `INSERT OR IGNORE INTO exercise_tags (exercise_name, kind, value)
     VALUES (?, ?, ?)`,
  );
  const tagDelete = env.SITE_DB.prepare(
    "DELETE FROM exercise_tags WHERE exercise_name = ?",
  );
  for (const exercise of parsed.exercises) {
    statements.push(exerciseInsert.bind(exercise.name));
    if (changedExercises.has(exercise.name)) {
      statements.push(tagDelete.bind(exercise.name));
      for (const tag of exercise.tags) {
        statements.push(tagInsert.bind(exercise.name, tag.kind, tag.value));
      }
    }
  }

  const setInsert = env.SITE_DB.prepare(
    `INSERT INTO sets
       (id, workout_id, exercise_name, raw_exercise_name, ordinal,
        exercise_note, superset_id, weight_milli, reps, effort_hundredths,
        distance_milli, set_time_seconds, set_type, incomplete)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
     ON CONFLICT(id) DO NOTHING`,
  );
  const recordInsert = env.SITE_DB.prepare(
    `INSERT OR IGNORE INTO set_records (set_id, ordinal, level, kind)
     VALUES (?, ?, ?, ?)`,
  );
  const setStatementIndexes: number[] = [];
  for (const set of candidates) {
    setStatementIndexes.push(statements.length);
    statements.push(
      setInsert.bind(
        set.id,
        set.workout_id,
        set.exercise_name,
        set.raw_exercise_name,
        set.ordinal,
        set.exercise_note,
        set.superset_id,
        set.weight_milli,
        set.reps,
        set.effort_hundredths,
        set.distance_milli,
        set.set_time_seconds,
        set.set_type,
        set.reps === null && set.distance_milli === null && set.set_time_seconds === null
          ? 1
          : 0,
      ),
    );
    set.records.forEach((record, index) => {
      statements.push(
        recordInsert.bind(set.id, index + 1, record.level, record.kind),
      );
    });
  }
  statements.push(
    env.SITE_DB.prepare(
      "UPDATE fitness_meta SET v = v + 1 WHERE k = 'version'",
    ),
  );

  const outcomes = await env.SITE_DB.batch(statements);
  const added = setStatementIndexes.reduce(
    (sum, index) => sum + (outcomes[index]?.meta?.changes ?? 0),
    0,
  );
  return json({
    received: parsed.sets.length,
    added,
    skipped: parsed.sets.length - added,
    version: await fitnessDataVersion(env),
  });
}

function parseFilters(params: URLSearchParams): FilterSql | string {
  for (const key of params.keys()) {
    if (!ALLOWED_FILTERS.has(key)) return `unknown filter: ${key}`;
  }
  const conditions: string[] = ["1 = 1"];
  const values: Array<string | number> = [];

  const q = singleValue(params, "q");
  if (q === DUPLICATE_PARAM) return "q may appear only once";
  if (q !== null) {
    const term = q.trim();
    if (!validText(term, 1, 100)) return "q must be 1-100 text characters";
    const like = `%${escapeLike(term)}%`;
    // D1 caps LIKE/GLOB patterns at 50 UTF-8 bytes. Check the final escaped
    // pattern so multibyte text and literal %, _, or \\ cannot turn into a 500.
    if (new TextEncoder().encode(like).byteLength > 50) {
      return "q is too long after escaping (50-byte pattern limit)";
    }
    conditions.push(
      `(w.title LIKE ? ESCAPE '\\' COLLATE NOCASE
        OR w.raw_title LIKE ? ESCAPE '\\' COLLATE NOCASE
        OR w.notes LIKE ? ESCAPE '\\' COLLATE NOCASE
        OR w.description LIKE ? ESCAPE '\\' COLLATE NOCASE
        OR e.name LIKE ? ESCAPE '\\' COLLATE NOCASE
        OR s.raw_exercise_name LIKE ? ESCAPE '\\' COLLATE NOCASE
        OR s.exercise_note LIKE ? ESCAPE '\\' COLLATE NOCASE)`,
    );
    values.push(like, like, like, like, like, like, like);
  }

  for (const kind of TAG_KINDS) {
    const selected = repeatedValues(params, kind, TAG_PATTERN, 8);
    if (typeof selected === "string") return selected;
    if (selected.length > 0) {
      conditions.push(
        `EXISTS (
          SELECT 1 FROM exercise_tags et_${kind}
           WHERE et_${kind}.exercise_name = e.name
             AND et_${kind}.kind = '${kind}'
             AND et_${kind}.value IN (${placeholders(selected.length)})
        )`,
      );
      values.push(...selected);
    }
  }

  const setTypes = repeatedValues(params, "set_type", SET_TYPE_PATTERN, 8);
  if (typeof setTypes === "string") return setTypes;
  if (setTypes.length > 0) {
    conditions.push(`s.set_type IN (${placeholders(setTypes.length)})`);
    values.push(...setTypes);
  }

  const exercise = singleValue(params, "exercise");
  if (exercise === DUPLICATE_PARAM) return "exercise may appear only once";
  if (exercise !== null) {
    if (!validText(exercise, 1, 240) || exercise.trim().length === 0) {
      return "exercise must be 1-240 non-whitespace characters";
    }
    conditions.push("e.name = ? COLLATE NOCASE");
    values.push(exercise);
  }

  const from = singleValue(params, "from");
  if (from === DUPLICATE_PARAM) return "from may appear only once";
  if (from !== null) {
    if (!validDate(from)) return "from must be a real YYYY-MM-DD date";
    conditions.push("substr(w.started_at_local, 1, 10) >= ?");
    values.push(from);
  }
  const to = singleValue(params, "to");
  if (to === DUPLICATE_PARAM) return "to may appear only once";
  if (to !== null) {
    if (!validDate(to)) return "to must be a real YYYY-MM-DD date";
    conditions.push("substr(w.started_at_local, 1, 10) <= ?");
    values.push(to);
  }
  if (from !== null && to !== null && from > to) return "from must not exceed to";

  const time = singleValue(params, "time_of_day");
  if (time === DUPLICATE_PARAM) return "time_of_day may appear only once";
  if (time !== null) {
    if (!isTimeOfDay(time)) {
      return "time_of_day must be morning, afternoon, evening, or night";
    }
    const hour = "CAST(substr(w.started_at_local, 12, 2) AS INTEGER)";
    const byTime: Record<TimeOfDay, string> = {
      morning: `${hour} BETWEEN 5 AND 11`,
      afternoon: `${hour} BETWEEN 12 AND 16`,
      evening: `${hour} BETWEEN 17 AND 20`,
      night: `(${hour} >= 21 OR ${hour} <= 4)`,
    };
    conditions.push(byTime[time]);
  }

  const weekday = singleValue(params, "weekday");
  if (weekday === DUPLICATE_PARAM) return "weekday may appear only once";
  if (weekday !== null) {
    const weekdays: Record<string, number> = {
      sun: 0,
      mon: 1,
      tue: 2,
      wed: 3,
      thu: 4,
      fri: 5,
      sat: 6,
    };
    const day = weekdays[weekday];
    if (day === undefined) return "weekday must be sun, mon, tue, wed, thu, fri, or sat";
    conditions.push("CAST(strftime('%w', w.started_at_local) AS INTEGER) = ?");
    values.push(day);
  }

  const minLoad = scaledFilter(params, "min_load", 3, 1_000_000_000);
  if (typeof minLoad === "string") return minLoad;
  if (minLoad !== null) {
    conditions.push("s.weight_milli >= ?");
    values.push(minLoad);
  }
  const maxLoad = scaledFilter(params, "max_load", 3, 1_000_000_000);
  if (typeof maxLoad === "string") return maxLoad;
  if (maxLoad !== null) {
    conditions.push("s.weight_milli <= ?");
    values.push(maxLoad);
  }
  if (minLoad !== null && maxLoad !== null && minLoad > maxLoad) {
    return "min_load must not exceed max_load";
  }

  const minReps = integerFilter(params, "min_reps", 0, 1_000_000);
  if (typeof minReps === "string") return minReps;
  if (minReps !== null) {
    conditions.push("s.reps >= ?");
    values.push(minReps);
  }
  const maxReps = integerFilter(params, "max_reps", 0, 1_000_000);
  if (typeof maxReps === "string") return maxReps;
  if (maxReps !== null) {
    conditions.push("s.reps <= ?");
    values.push(maxReps);
  }
  if (minReps !== null && maxReps !== null && minReps > maxReps) {
    return "min_reps must not exceed max_reps";
  }

  const maxEffort = scaledFilter(params, "max_effort", 2, 100_000);
  if (typeof maxEffort === "string") return maxEffort;
  if (maxEffort !== null) {
    conditions.push("s.effort_hundredths <= ?");
    values.push(maxEffort);
  }

  for (const [key, sql] of [
    ["has_record", "EXISTS (SELECT 1 FROM set_records sr WHERE sr.set_id = s.id)"],
    ["has_superset", "s.superset_id IS NOT NULL"],
    [
      "has_notes",
      "(w.notes IS NOT NULL OR w.description IS NOT NULL OR s.exercise_note IS NOT NULL)",
    ],
  ]) {
    const flag = booleanFilter(params, key);
    if (typeof flag === "string") return flag;
    if (flag !== null) conditions.push(flag ? sql : `NOT (${sql})`);
  }

  const incomplete = booleanFilter(params, "incomplete");
  if (typeof incomplete === "string") return incomplete;
  if (incomplete !== null) {
    conditions.push("s.incomplete = ?");
    values.push(incomplete ? 1 : 0);
  }

  const duration = singleValue(params, "duration");
  if (duration === DUPLICATE_PARAM) return "duration may appear only once";
  if (duration !== null) {
    if (duration !== "normal" && duration !== "suspicious") {
      return "duration must be normal or suspicious";
    }
    conditions.push("w.duration_suspicious = ?");
    values.push(duration === "suspicious" ? 1 : 0);
  }

  const page = integerFilter(params, "page", 1, 100_000);
  if (typeof page === "string") return page;
  const perPageValue = integerFilter(params, "per_page", 10, 40);
  if (typeof perPageValue === "string") return perPageValue;
  const perPage = perPageValue ?? 20;
  if (perPage !== 10 && perPage !== 20 && perPage !== 40) {
    return "per_page must be 10, 20, or 40";
  }

  return {
    where: conditions.join(" AND "),
    params: values,
    page: page ?? 1,
    perPage,
  };
}

function parseImportPayload(value: unknown): ImportPayload | string {
  if (!isObject(value)) return "body must be an object";
  if (!hasOnlyKeys(value, ["workouts", "exercises", "sets"])) {
    return "body may contain only workouts, exercises, and sets";
  }
  if (!Array.isArray(value.workouts)) return "workouts must be an array";
  if (!Array.isArray(value.exercises)) return "exercises must be an array";
  if (!Array.isArray(value.sets)) return "sets must be an array";
  if (value.workouts.length < 1 || value.workouts.length > MAX_IMPORT_WORKOUTS) {
    return `workouts must contain 1-${MAX_IMPORT_WORKOUTS} entries`;
  }
  if (value.exercises.length < 1 || value.exercises.length > MAX_IMPORT_EXERCISES) {
    return `exercises must contain 1-${MAX_IMPORT_EXERCISES} entries`;
  }
  if (value.sets.length < 1 || value.sets.length > MAX_IMPORT_SETS) {
    return `sets must contain 1-${MAX_IMPORT_SETS} entries`;
  }

  const workouts: IncomingWorkout[] = [];
  for (let index = 0; index < value.workouts.length; index += 1) {
    const workout = parseWorkout(value.workouts[index]);
    if (typeof workout === "string") return `workouts[${index}]: ${workout}`;
    workouts.push(workout);
  }
  const exercises: IncomingExercise[] = [];
  for (let index = 0; index < value.exercises.length; index += 1) {
    const exercise = parseExercise(value.exercises[index]);
    if (typeof exercise === "string") return `exercises[${index}]: ${exercise}`;
    exercises.push(exercise);
  }
  const sets: IncomingSet[] = [];
  for (let index = 0; index < value.sets.length; index += 1) {
    const set = parseSet(value.sets[index]);
    if (typeof set === "string") return `sets[${index}]: ${set}`;
    sets.push(set);
  }

  const duplicateWorkout = duplicateId(workouts);
  if (duplicateWorkout) return `duplicate workout id: ${duplicateWorkout}`;
  const duplicateExercise = duplicateValue(exercises.map((exercise) => exercise.name));
  if (duplicateExercise) return `duplicate exercise name: ${duplicateExercise}`;
  const duplicateSet = duplicateId(sets);
  if (duplicateSet) return `duplicate set id: ${duplicateSet}`;

  const workoutIds = new Set(workouts.map((workout) => workout.id));
  const exerciseNames = new Set(exercises.map((exercise) => exercise.name));
  const ordinals = new Set<string>();
  for (const set of sets) {
    if (!workoutIds.has(set.workout_id)) {
      return `set ${set.id} references a workout absent from this chunk`;
    }
    if (!exerciseNames.has(set.exercise_name)) {
      return `set ${set.id} references an exercise absent from this chunk`;
    }
    const ordinalKey = `${set.workout_id}\0${set.ordinal}`;
    if (ordinals.has(ordinalKey)) {
      return `duplicate ordinal ${set.ordinal} in workout ${set.workout_id}`;
    }
    ordinals.add(ordinalKey);
  }

  const totalTags = exercises.reduce(
    (sum, exercise) => sum + exercise.tags.length,
    0,
  );
  if (totalTags > MAX_TOTAL_TAGS) return `chunk may contain at most ${MAX_TOTAL_TAGS} tags`;
  const totalRecords = sets.reduce((sum, set) => sum + set.records.length, 0);
  if (totalRecords > MAX_TOTAL_RECORDS) {
    return `chunk may contain at most ${MAX_TOTAL_RECORDS} records`;
  }
  return { workouts, exercises, sets };
}

function parseWorkout(value: unknown): IncomingWorkout | string {
  if (!isObject(value)) return "must be an object";
  if (
    !hasOnlyKeys(value, [
      "id",
      "title",
      "raw_title",
      "started_at_local",
      "duration_seconds",
      "duration_suspicious",
      "notes",
      "description",
      "source",
    ])
  ) {
    return "contains unknown or missing fields";
  }
  const { id, title, raw_title, started_at_local, duration_seconds, duration_suspicious } =
    value;
  if (typeof id !== "string" || !ID_PATTERN.test(id)) return "bad id";
  if (!validText(title, 1, 240) || title.trim().length === 0) {
    return "title must be 1-240 non-whitespace characters";
  }
  if (!validText(raw_title, 1, 240)) return "raw_title must be 1-240 characters";
  if (typeof started_at_local !== "string" || !validLocalDateTime(started_at_local)) {
    return "started_at_local must be a real YYYY-MM-DD HH:MM:SS local time";
  }
  if (!validInteger(duration_seconds, 0, 604_800)) return "bad duration_seconds";
  if (typeof duration_suspicious !== "boolean") return "bad duration_suspicious";
  if (duration_suspicious !== (duration_seconds === 0 || duration_seconds >= 14_400)) {
    return "duration_suspicious does not match duration_seconds";
  }
  const notes = nullableText(value.notes, 10_000);
  if (notes === undefined) return "notes must be null or 1-10000 characters";
  const description = nullableText(value.description, 10_000);
  if (description === undefined) return "description must be null or 1-10000 characters";
  if (value.source !== "workout-data-csv") return "source must be workout-data-csv";
  return {
    id,
    title,
    raw_title,
    started_at_local,
    duration_seconds,
    duration_suspicious,
    notes,
    description,
    source: value.source,
  };
}

function parseExercise(value: unknown): IncomingExercise | string {
  if (!isObject(value)) return "must be an object";
  if (!hasOnlyKeys(value, ["name", "tags"])) {
    return "contains unknown or missing fields";
  }
  const { name, tags } = value;
  if (!validText(name, 1, 240) || name.trim().length === 0) {
    return "name must be 1-240 non-whitespace characters";
  }
  if (!Array.isArray(tags) || tags.length > 60) {
    return "tags must be an array of at most 60 entries";
  }
  const parsedTags: IncomingTag[] = [];
  const seen = new Set<string>();
  for (let index = 0; index < tags.length; index += 1) {
    const tag = tags[index];
    if (!isObject(tag) || !hasOnlyKeys(tag, ["kind", "value"])) {
      return `tags[${index}] must contain only kind and value`;
    }
    if (typeof tag.kind !== "string" || !isTagKind(tag.kind)) {
      return `bad tags[${index}].kind`;
    }
    if (typeof tag.value !== "string" || !TAG_PATTERN.test(tag.value)) {
      return `bad tags[${index}].value`;
    }
    const key = `${tag.kind}\0${tag.value}`;
    if (seen.has(key)) return `duplicate tag: ${tag.kind}/${tag.value}`;
    seen.add(key);
    parsedTags.push({ kind: tag.kind, value: tag.value });
  }
  return { name, tags: parsedTags };
}

function parseSet(value: unknown): IncomingSet | string {
  if (!isObject(value)) return "must be an object";
  if (
    !hasOnlyKeys(value, [
      "id",
      "workout_id",
      "ordinal",
      "exercise_name",
      "raw_exercise_name",
      "exercise_note",
      "superset_id",
      "weight_milli",
      "reps",
      "effort_hundredths",
      "distance_milli",
      "set_time_seconds",
      "set_type",
      "records",
    ])
  ) {
    return "contains unknown or missing fields";
  }
  const { id, workout_id, ordinal, exercise_name, raw_exercise_name, set_type, records } =
    value;
  if (typeof id !== "string" || !ID_PATTERN.test(id)) return "bad id";
  if (typeof workout_id !== "string" || !ID_PATTERN.test(workout_id)) return "bad workout_id";
  if (!validInteger(ordinal, 1, 10_000)) return "bad ordinal";
  if (!validText(exercise_name, 1, 240)) return "bad exercise_name";
  if (!validText(raw_exercise_name, 1, 240)) return "bad raw_exercise_name";
  const exercise_note = nullableText(value.exercise_note, 2_000);
  if (exercise_note === undefined) return "bad exercise_note";
  const superset_id = nullableInteger(value.superset_id, 0, 1_000_000);
  if (superset_id === undefined) return "bad superset_id";
  const weight_milli = nullableInteger(value.weight_milli, 0, 1_000_000_000);
  if (weight_milli === undefined) return "bad weight_milli";
  const reps = nullableInteger(value.reps, 0, 1_000_000);
  if (reps === undefined) return "bad reps";
  const effort_hundredths = nullableInteger(value.effort_hundredths, 0, 100_000);
  if (effort_hundredths === undefined) return "bad effort_hundredths";
  const distance_milli = nullableInteger(value.distance_milli, 0, 1_000_000_000);
  if (distance_milli === undefined) return "bad distance_milli";
  const set_time_seconds = nullableInteger(value.set_time_seconds, 0, 604_800);
  if (set_time_seconds === undefined) return "bad set_time_seconds";
  if (typeof set_type !== "string" || !SET_TYPE_PATTERN.test(set_type)) return "bad set_type";
  if (!Array.isArray(records) || records.length > 20) {
    return "records must be an array of at most 20 entries";
  }
  const parsedRecords: IncomingRecord[] = [];
  const recordKinds = new Set<string>();
  for (let index = 0; index < records.length; index += 1) {
    const record = parseRecord(records[index]);
    if (typeof record === "string") return `records[${index}]: ${record}`;
    if (recordKinds.has(record.kind)) return `duplicate record kind: ${record.kind}`;
    recordKinds.add(record.kind);
    parsedRecords.push(record);
  }
  return {
    id,
    workout_id,
    ordinal,
    exercise_name,
    raw_exercise_name,
    exercise_note,
    superset_id,
    weight_milli,
    reps,
    effort_hundredths,
    distance_milli,
    set_time_seconds,
    set_type,
    records: parsedRecords,
  };
}

function parseRecord(value: unknown): IncomingRecord | string {
  if (!isObject(value) || !hasOnlyKeys(value, ["level", "kind"])) {
    return "must contain only level and kind";
  }
  const { level, kind } = value;
  if (level !== "gold" && level !== "silver" && level !== "bronze") {
    return "bad level";
  }
  if (kind !== "1rm" && kind !== "max-weight" && kind !== "volume" && kind !== "reps") {
    return "bad kind";
  }
  return { level, kind };
}

async function readJson(request: Request): Promise<unknown | string> {
  const declared = request.headers.get("Content-Length");
  if (declared !== null) {
    const length = Number(declared);
    if (!Number.isInteger(length) || length < 0) return "bad Content-Length";
    if (length > BODY_LIMIT_BYTES) return `body exceeds ${BODY_LIMIT_BYTES} bytes`;
  }
  if (!request.body) return "body must be JSON";

  const reader = request.body.getReader();
  const decoder = new TextDecoder();
  let bytes = 0;
  let text = "";
  while (true) {
    const chunk = await reader.read();
    if (chunk.done) break;
    bytes += chunk.value.byteLength;
    if (bytes > BODY_LIMIT_BYTES) {
      await reader.cancel("request body too large");
      return `body exceeds ${BODY_LIMIT_BYTES} bytes`;
    }
    text += decoder.decode(chunk.value, { stream: true });
  }
  text += decoder.decode();
  try {
    const parsed: unknown = JSON.parse(text);
    return parsed;
  } catch {
    return "body must be JSON";
  }
}

function singleValue(
  params: URLSearchParams,
  key: string,
): string | null | typeof DUPLICATE_PARAM {
  const values = params.getAll(key);
  if (values.length > 1) return DUPLICATE_PARAM;
  return values[0] ?? null;
}

function repeatedValues(
  params: URLSearchParams,
  key: string,
  pattern: RegExp,
  limit: number,
): string[] | string {
  const entries = params.getAll(key);
  if (entries.length > limit) return `${key} may appear at most ${limit} times`;
  const unique = new Set<string>();
  for (const entry of entries) {
    if (!pattern.test(entry)) return `bad ${key} value`;
    if (unique.has(entry)) return `duplicate ${key} value: ${entry}`;
    unique.add(entry);
  }
  return entries;
}

function scaledFilter(
  params: URLSearchParams,
  key: string,
  places: number,
  maxScaled: number,
): number | null | string {
  const value = singleValue(params, key);
  if (value === DUPLICATE_PARAM) return `${key} may appear only once`;
  if (value === null) return null;
  const scaled = scaledDecimal(value, places);
  if (scaled === null || scaled > maxScaled) {
    return `${key} must be a non-negative decimal with at most ${places} places`;
  }
  return scaled;
}

function integerFilter(
  params: URLSearchParams,
  key: string,
  min: number,
  max: number,
): number | null | string {
  const value = singleValue(params, key);
  if (value === DUPLICATE_PARAM) return `${key} may appear only once`;
  if (value === null) return null;
  if (!/^(0|[1-9]\d*)$/.test(value)) return `${key} must be an integer`;
  const parsed = Number(value);
  if (!validInteger(parsed, min, max)) return `${key} must be between ${min} and ${max}`;
  return parsed;
}

function booleanFilter(
  params: URLSearchParams,
  key: string,
): boolean | null | string {
  const value = singleValue(params, key);
  if (value === DUPLICATE_PARAM) return `${key} may appear only once`;
  if (value === null) return null;
  if (value === "true") return true;
  if (value === "false") return false;
  return `${key} must be true or false`;
}

function scaledDecimal(value: string, places: number): number | null {
  const match = value.match(/^(0|[1-9]\d{0,8})(?:\.(\d+))?$/);
  if (!match || (match[2]?.length ?? 0) > places) return null;
  const fraction = (match[2] ?? "").padEnd(places, "0");
  const scale = 10 ** places;
  const result = Number(match[1]) * scale + Number(fraction || "0");
  return Number.isSafeInteger(result) ? result : null;
}

function validDate(value: string): boolean {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!match) return false;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(Date.UTC(year, month - 1, day));
  return (
    date.getUTCFullYear() === year &&
    date.getUTCMonth() === month - 1 &&
    date.getUTCDate() === day
  );
}

function validLocalDateTime(value: string): boolean {
  const match = value.match(/^(\d{4}-\d{2}-\d{2}) (\d{2}):(\d{2}):(\d{2})$/);
  if (!match || !validDate(match[1])) return false;
  return Number(match[2]) <= 23 && Number(match[3]) <= 59 && Number(match[4]) <= 59;
}

function validInteger(value: unknown, min: number, max: number): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= min && value <= max;
}

function nullableInteger(
  value: unknown,
  min: number,
  max: number,
): number | null | undefined {
  if (value === null) return null;
  return validInteger(value, min, max) ? value : undefined;
}

function nullableText(value: unknown, max: number): string | null | undefined {
  if (value === null) return null;
  return validText(value, 1, max) ? value : undefined;
}

function validText(value: unknown, min: number, max: number): value is string {
  return (
    typeof value === "string" &&
    value.length >= min &&
    value.length <= max &&
    !/[\u0000-\u0008\u000b\u000c\u000e-\u001f]/.test(value)
  );
}

function duplicateId(values: Array<{ id: string }>): string | null {
  const seen = new Set<string>();
  for (const value of values) {
    if (seen.has(value.id)) return value.id;
    seen.add(value.id);
  }
  return null;
}

function duplicateValue(values: string[]): string | null {
  const seen = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) return value;
    seen.add(value);
  }
  return null;
}

function tagSignature(tags: IncomingTag[]): string {
  return tags
    .map((tag) => `${tag.kind}\0${tag.value}`)
    .sort()
    .join("\u0001");
}

function hasOnlyKeys(value: Record<string, unknown>, keys: readonly string[]): boolean {
  const actual = Object.keys(value);
  return actual.length === keys.length && keys.every((key) => Object.hasOwn(value, key));
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isTagKind(value: string): value is TagKind {
  return value === "movement" || value === "muscle" || value === "equipment";
}

function isTimeOfDay(value: string): value is TimeOfDay {
  return value === "morning" || value === "afternoon" || value === "evening" || value === "night";
}

function placeholders(count: number): string {
  return Array.from({ length: count }, () => "?").join(", ");
}

function escapeLike(value: string): string {
  return value.replace(/[\\%_]/g, "\\$&");
}

function badFilter(message: string): Response {
  return json({ error: message }, 400, true);
}

function json(payload: unknown, status = 200, publicRead = false): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: {
      "Content-Type": "application/json; charset=utf-8",
      "Cache-Control": "no-store",
      ...(publicRead ? PUBLIC_HEADERS : {}),
    },
  });
}
