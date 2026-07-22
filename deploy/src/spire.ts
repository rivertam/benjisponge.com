// /api/spire/* — the Slay the Spire 2 run database, backed by D1. The GET
// endpoints are public (the container renders /spire, the homepage, and the
// feed from them); POST is the sync CLI's write path, bearer-authed with the
// SPIRE_SYNC_TOKEN secret. Responses are no-store: freshness levers for the
// rendered pages live in cache.ts, not here.

import { bearerAuthorized } from "./auth";

type SpireEnv = Env & { SPIRE_SYNC_TOKEN?: string };

const COLUMNS =
  "id, date, start_time, character, win, abandoned, ascension, acts, " +
  "floors, killed_by, kill_kind, run_time, seed, game_mode, build_id";

// A stored row; win/abandoned are 0/1 in SQLite and booleans on the wire.
type RunRow = {
  id: string;
  date: string;
  start_time: number;
  character: string;
  win: number;
  abandoned: number;
  ascension: number;
  acts: number;
  floors: number;
  killed_by: string | null;
  kill_kind: string | null;
  run_time: number;
  seed: string;
  game_mode: string;
  build_id: string;
};

// What the sync CLI uploads: RunRow with real booleans plus the whole
// original .run file, kept so future redesigns never need a re-scrape.
type IncomingRun = Omit<RunRow, "win" | "abandoned"> & {
  win: boolean;
  abandoned: boolean;
  raw: string;
};

export async function handleSpire(
  request: Request,
  env: SpireEnv,
  url: URL,
): Promise<Response> {
  try {
    if (request.method === "GET" && url.pathname === "/api/spire/runs") {
      return await listRuns(env);
    }
    if (request.method === "GET" && url.pathname === "/api/spire/ids") {
      return await listIds(env);
    }
    if (request.method === "POST" && url.pathname === "/api/spire/runs") {
      return await insertRuns(request, env);
    }
  } catch (err) {
    console.error(
      JSON.stringify({
        message: "spire api failed",
        path: url.pathname,
        error: err instanceof Error ? err.message : String(err),
      }),
    );
    return json({ error: "internal error" }, 500);
  }
  return json({ error: "not found" }, 404);
}

// The data version pages embed in their edge-cache key (see cache.ts). Null
// on any failure — the caller falls back to an unversioned key rather than
// failing the page.
export async function spireDataVersion(env: Env): Promise<number | null> {
  try {
    const version = await env.SITE_DB.prepare(
      "SELECT v FROM spire_meta WHERE k = 'version'",
    ).first<number>("v");
    return version ?? 0;
  } catch (err) {
    console.error(
      JSON.stringify({
        message: "spire version lookup failed",
        error: err instanceof Error ? err.message : String(err),
      }),
    );
    return null;
  }
}

async function listRuns(env: SpireEnv): Promise<Response> {
  const { results } = await env.SITE_DB.prepare(
    `SELECT ${COLUMNS} FROM spire_runs ORDER BY start_time DESC`,
  ).all<RunRow>();
  const runs = results.map((row) => ({
    ...row,
    win: row.win === 1,
    abandoned: row.abandoned === 1,
  }));
  const version = (await spireDataVersion(env)) ?? 0;
  return json({ version, count: runs.length, runs });
}

async function listIds(env: SpireEnv): Promise<Response> {
  const { results } = await env.SITE_DB.prepare(
    "SELECT id FROM spire_runs",
  ).all<{ id: string }>();
  return json({ ids: results.map((row) => row.id) });
}

async function insertRuns(request: Request, env: SpireEnv): Promise<Response> {
  if (!(await bearerAuthorized(request, env.SPIRE_SYNC_TOKEN))) {
    return json({ error: "unauthorized" }, 401);
  }
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return json({ error: "body must be JSON" }, 400);
  }
  if (!isObject(body)) return json({ error: "body must be an object" }, 400);
  const runs = body.runs;
  if (!Array.isArray(runs) || runs.length === 0 || runs.length > 50) {
    return json({ error: "runs must be an array of 1–50 entries" }, 400);
  }
  const rows: IncomingRun[] = [];
  for (const candidate of runs) {
    const parsed = parseRun(candidate);
    if (typeof parsed === "string") return json({ error: parsed }, 400);
    rows.push(parsed);
  }

  const insert = env.SITE_DB.prepare(
    `INSERT OR IGNORE INTO spire_runs (${COLUMNS}, raw, added_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, unixepoch())`,
  );
  const outcomes = await env.SITE_DB.batch(
    rows.map((r) =>
      insert.bind(
        r.id,
        r.date,
        r.start_time,
        r.character,
        r.win ? 1 : 0,
        r.abandoned ? 1 : 0,
        r.ascension,
        r.acts,
        r.floors,
        r.killed_by,
        r.kill_kind,
        r.run_time,
        r.seed,
        r.game_mode,
        r.build_id,
        r.raw,
      ),
    ),
  );
  const added = outcomes.reduce((sum, o) => sum + (o.meta?.changes ?? 0), 0);
  if (added > 0) {
    // Bumping the version is what invalidates the edge-cached pages.
    await env.SITE_DB.prepare(
      "UPDATE spire_meta SET v = v + 1 WHERE k = 'version'",
    ).run();
  }
  const version = (await spireDataVersion(env)) ?? 0;
  return json({
    received: rows.length,
    added,
    skipped: rows.length - added,
    version,
  });
}

function parseRun(value: unknown): IncomingRun | string {
  if (typeof value !== "object" || value === null) return "run must be an object";
  if (!isObject(value)) return "run must be an object";
  const {
    id,
    date,
    start_time,
    character,
    win,
    abandoned,
    ascension,
    acts,
    floors,
    killed_by,
    kill_kind,
    run_time,
    seed,
    game_mode,
    build_id,
    raw,
  } = value;
  if (typeof id !== "string" || !/^\d{1,12}$/.test(id)) return "bad id";
  if (typeof date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(date)) {
    return `bad date on run ${id}`;
  }
  if (!validInteger(start_time)) return `bad start_time on run ${id}`;
  if (!validInteger(ascension)) return `bad ascension on run ${id}`;
  if (!validInteger(acts)) return `bad acts on run ${id}`;
  if (!validInteger(floors)) return `bad floors on run ${id}`;
  if (!validInteger(run_time)) return `bad run_time on run ${id}`;
  if (typeof win !== "boolean" || typeof abandoned !== "boolean") {
    return `bad result flags on run ${id}`;
  }
  if (typeof character !== "string" || character.length === 0 || character.length > 120) {
    return `bad character on run ${id}`;
  }
  if (typeof seed !== "string" || seed.length > 120) return `bad seed on run ${id}`;
  if (typeof game_mode !== "string" || game_mode.length > 120) {
    return `bad game_mode on run ${id}`;
  }
  if (typeof build_id !== "string" || build_id.length > 120) {
    return `bad build_id on run ${id}`;
  }
  if (killed_by !== null && (typeof killed_by !== "string" || killed_by.length > 120)) {
    return `bad killed_by on run ${id}`;
  }
  if (kill_kind !== null && (typeof kill_kind !== "string" || kill_kind.length > 120)) {
    return `bad kill_kind on run ${id}`;
  }
  // Largest observed .run file is ~97 KB; 500 KB leaves generous headroom.
  if (typeof raw !== "string" || raw.length === 0 || raw.length > 500_000) {
    return `bad raw on run ${id}`;
  }
  return {
    id,
    date,
    start_time,
    character,
    win,
    abandoned,
    ascension,
    acts,
    floors,
    killed_by,
    kill_kind,
    run_time,
    seed,
    game_mode,
    build_id,
    raw,
  };
}

function validInteger(value: unknown): value is number {
  return (
    typeof value === "number" &&
    Number.isInteger(value) &&
    value >= 0 &&
    value <= 1e12
  );
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function json(payload: unknown, status = 200): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: {
      "Content-Type": "application/json; charset=utf-8",
      "Cache-Control": "no-store",
    },
  });
}
