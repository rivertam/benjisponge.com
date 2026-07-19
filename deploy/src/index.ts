import { Container, getContainer } from "@cloudflare/containers";
import { cacheKey, cacheable, fromCache, storeInCache } from "./cache";

export class BenjispongeContainer extends Container<Env> {
  defaultPort = 8080;
  sleepAfter = "15m";
  envVars = { SITE_ORIGIN: "https://benjisponge.com" };
}

export default {
  async fetch(request, env, ctx): Promise<Response> {
    const url = new URL(request.url);

    // Collapse www so the planes page's Host-derived QR URL has one origin.
    if (url.hostname === "www.benjisponge.com") {
      return Response.redirect(
        `https://benjisponge.com${url.pathname}${url.search}`,
        301,
      );
    }

    const container = getContainer(env.SITE_CONTAINER, "site");

    if (!cacheable(request, url)) {
      // Shard POSTs (/_topcoat/shards/*) and any future mutating routes.
      return container.fetch(request);
    }

    const key = cacheKey(url, env.RELEASE_ID);
    const hit = await fromCache(key);
    if (hit) return hit;

    const response = await container.fetch(request);
    if (!response.ok) return response; // 404s stay uncached — cheap and deterministic
    return storeInCache(ctx, key, response);
  },
} satisfies ExportedHandler<Env>;
