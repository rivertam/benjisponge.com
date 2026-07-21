// Edge-cache policy for rendered HTML. Hashed /_topcoat/assets/* files never
// reach the Worker (the static-asset layer serves them first), so this only
// governs page renders and the feed.
//
// The cache key embeds RELEASE_ID, so every deploy atomically invalidates all
// cached pages without purge API calls. When more dynamic content arrives
// (polls, etc.), add its purge lever here: either list routes in NEVER_CACHE,
// or add a data-version segment to the key and bump it on writes — the spire
// run log does the latter via DATA_VERSIONED.

const NEVER_CACHE: string[] = [];

// Pages that render the synced spire run log (exact pathnames). Their cache
// key embeds the run database's version counter, so a sync invalidates them
// on the next request — no deploy, no purge call.
export const DATA_VERSIONED = new Set(["/", "/spire", "/feed.xml"]);

// The Worker adds this only when it sends a data-versioned cache miss to the
// container. The renderers use it to bypass their one-minute run-data cache,
// ensuring the new edge-cache entry contains the newly synced run.
export const SPIRE_CACHE_REFRESH_HEADER = "x-spire-cache-refresh";

export function cacheable(request: Request, url: URL): boolean {
  if (request.method !== "GET") return false;
  return !NEVER_CACHE.some((prefix) => url.pathname.startsWith(prefix));
}

export function cacheKey(
  url: URL,
  releaseId: string,
  dataVersion: number | null = null,
): Request {
  // Synthetic host: keys must be valid URLs, and this can never collide with a
  // real cached resource. A null dataVersion (page doesn't render spire data,
  // or the version lookup failed) leaves the key purely release-scoped.
  const data = dataVersion === null ? "" : `.d${dataVersion}`;
  return new Request(
    `https://edge-cache.invalid/${releaseId}${data}${url.pathname}${url.search}`,
  );
}

export async function fromCache(key: Request): Promise<Response | undefined> {
  return caches.default.match(key);
}

export function refreshSpireData(request: Request): Request {
  const headers = new Headers(request.headers);
  headers.set(SPIRE_CACHE_REFRESH_HEADER, "1");
  return new Request(request, { headers });
}

export function storeInCache(
  ctx: ExecutionContext,
  key: Request,
  response: Response,
): Response {
  const copy = new Response(response.clone().body, response);
  // s-maxage governs the edge; max-age=0 keeps browsers revalidating so a
  // deploy shows up on the next page load.
  copy.headers.set("Cache-Control", "public, max-age=0, s-maxage=86400");
  ctx.waitUntil(caches.default.put(key, copy.clone()));
  return copy;
}
