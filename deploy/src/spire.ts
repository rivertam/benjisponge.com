// /api/spire/* — the Slay the Spire 2 run database, backed by D1. The GET
// endpoints are public (the container renders /spire, the homepage, and the
// feed from them); POST is the sync CLI's write path, bearer-authed with the
// SPIRE_SYNC_TOKEN secret. Responses are no-store: freshness levers for the
// rendered pages live in cache.ts, not here.

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
    console.error("spire api:", err);
    return json({ error: "internal error" }, 500);
  }
  return json({ error: "not found" }, 404);
}

// The data version pages embed in their edge-cache key (see cache.ts). Null
// on any failure — the caller falls back to an unversioned key rather than
// failing the page.
export async function spireDataVersion(env: Env): Promise<number | null> {
  try {
    const version = await env.SPIRE_DB.prepare(
      "SELECT v FROM spire_meta WHERE k = 'version'",
    ).first<number>("v");
    return version ?? 0;
  } catch (err) {
    console.error("spire version lookup failed:", err);
    return null;
  }
}

async function listRuns(env: SpireEnv): Promise<Response> {
  const { results } = await env.SPIRE_DB.prepare(
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
  const { results } = await env.SPIRE_DB.prepare(
    "SELECT id FROM spire_runs",
  ).all<{ id: string }>();
  return json({ ids: results.map((row) => row.id) });
}

async function insertRuns(request: Request, env: SpireEnv): Promise<Response> {
  if (!(await authorized(request, env))) {
    return json({ error: "unauthorized" }, 401);
  }
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return json({ error: "body must be JSON" }, 400);
  }
  const runs = (body as { runs?: unknown }).runs;
  if (!Array.isArray(runs) || runs.length === 0 || runs.length > 50) {
    return json({ error: "runs must be an array of 1–50 entries" }, 400);
  }
  const rows: IncomingRun[] = [];
  for (const candidate of runs) {
    const problem = validate(candidate);
    if (problem) return json({ error: problem }, 400);
    rows.push(candidate as IncomingRun);
  }

  const insert = env.SPIRE_DB.prepare(
    `INSERT OR IGNORE INTO spire_runs (${COLUMNS}, raw, added_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, unixepoch())`,
  );
  const outcomes = await env.SPIRE_DB.batch(
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
    await env.SPIRE_DB.prepare(
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

function validate(value: unknown): string | null {
  if (typeof value !== "object" || value === null) return "run must be an object";
  const r = value as Record<string, unknown>;
  if (typeof r.id !== "string" || !/^\d{1,12}$/.test(r.id)) return "bad id";
  if (typeof r.date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(r.date)) {
    return `bad date on run ${r.id}`;
  }
  for (const key of ["start_time", "ascension", "acts", "floors", "run_time"]) {
    const n = r[key];
    if (typeof n !== "number" || !Number.isInteger(n) || n < 0 || n > 1e12) {
      return `bad ${key} on run ${r.id}`;
    }
  }
  if (typeof r.win !== "boolean" || typeof r.abandoned !== "boolean") {
    return `bad result flags on run ${r.id}`;
  }
  if (typeof r.character !== "string" || r.character.length === 0 || r.character.length > 120) {
    return `bad character on run ${r.id}`;
  }
  for (const key of ["seed", "game_mode", "build_id"]) {
    const s = r[key];
    if (typeof s !== "string" || s.length > 120) return `bad ${key} on run ${r.id}`;
  }
  for (const key of ["killed_by", "kill_kind"]) {
    const s = r[key];
    if (s !== null && (typeof s !== "string" || s.length > 120)) {
      return `bad ${key} on run ${r.id}`;
    }
  }
  // Largest observed .run file is ~97 KB; 500 KB leaves generous headroom.
  if (typeof r.raw !== "string" || r.raw.length === 0 || r.raw.length > 500_000) {
    return `bad raw on run ${r.id}`;
  }
  return null;
}

async function authorized(request: Request, env: SpireEnv): Promise<boolean> {
  const expected = env.SPIRE_SYNC_TOKEN;
  if (!expected) return false; // secret unset → the write path stays closed
  const header = request.headers.get("Authorization") ?? "";
  const token = header.startsWith("Bearer ") ? header.slice(7).trim() : "";
  if (!token) return false;
  const enc = new TextEncoder();
  const a = enc.encode(token);
  const b = enc.encode(expected);
  if (a.byteLength !== b.byteLength) return false;
  return crypto.subtle.timingSafeEqual(a, b);
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
