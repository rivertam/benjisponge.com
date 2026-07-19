// Edge-cache policy for rendered HTML. Hashed /_topcoat/assets/* files never
// reach the Worker (the static-asset layer serves them first), so this only
// governs page renders and the feed.
//
// The cache key embeds RELEASE_ID, so every deploy atomically invalidates all
// cached pages without purge API calls. When dynamic content arrives (polls,
// etc.), add its purge lever here: either list routes in NEVER_CACHE, or add a
// data-version segment to the key (e.g. from KV) and bump it on writes.

const NEVER_CACHE: string[] = [];

export function cacheable(request: Request, url: URL): boolean {
  if (request.method !== "GET") return false;
  return !NEVER_CACHE.some((prefix) => url.pathname.startsWith(prefix));
}

export function cacheKey(url: URL, releaseId: string): Request {
  // Synthetic host: keys must be valid URLs, and this can never collide with a
  // real cached resource.
  return new Request(
    `https://edge-cache.invalid/${releaseId}${url.pathname}${url.search}`,
  );
}

export async function fromCache(key: Request): Promise<Response | undefined> {
  return caches.default.match(key);
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
