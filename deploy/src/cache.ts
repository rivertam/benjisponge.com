// Edge-cache policy for rendered HTML. Hashed /_topcoat/assets/* files never
// reach the Worker (the static-asset layer serves them first), so this only
// governs page renders and the feed.
//
// The cache key embeds RELEASE_ID, so every deploy atomically invalidates all
// cached pages without purge API calls. Freshness beyond that belongs to the
// renderers: a page that returns its own `s-maxage` keeps it (the spire pages
// use `public, max-age=0, s-maxage=60`, so a sync is visible within a
// minute), `no-store`/`private` bypasses the edge entirely, and everything
// else is treated as immutable within a release.

export function cacheable(request: Request): boolean {
  return request.method === "GET";
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
  if (bypassesSharedCache(response)) return response;

  const copy = new Response(response.clone().body, response);
  if (!hasSharedMaxAge(response)) {
    // s-maxage governs the edge; max-age=0 keeps browsers revalidating so a
    // deploy shows up on the next page load.
    copy.headers.set("Cache-Control", "public, max-age=0, s-maxage=86400");
  }
  ctx.waitUntil(caches.default.put(key, copy.clone()));
  return copy;
}

function directives(response: Response): string[] {
  const cacheControl = response.headers.get("Cache-Control");
  if (cacheControl === null) return [];
  return cacheControl
    .split(",")
    .map((directive) => directive.split("=", 1)[0].trim().toLowerCase());
}

function bypassesSharedCache(response: Response): boolean {
  return directives(response).some(
    (name) => name === "no-store" || name === "private",
  );
}

function hasSharedMaxAge(response: Response): boolean {
  return directives(response).includes("s-maxage");
}
