import { Container, getContainer } from "@cloudflare/containers";
import {
  DATA_VERSIONED,
  cacheKey,
  cacheable,
  fromCache,
  refreshSpireData,
  storeInCache,
} from "./cache";
import { handleFitness } from "./fitness";
import { handleSpire, spireDataVersion } from "./spire";

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

    // The spire run database API — served from D1 right here; never cached,
    // never touches the container (which is itself a GET-side consumer).
    if (url.pathname.startsWith("/api/spire")) {
      return handleSpire(request, env, url);
    }

    // Public fitness reads and the private bounded CSV import live in the
    // shared site D1 database and never touch the container.
    if (url.pathname.startsWith("/api/fitness")) {
      return handleFitness(request, env, url);
    }

    const container = getContainer(env.SITE_CONTAINER, "site");

    if (!cacheable(request, url)) {
      // Shard POSTs (/_topcoat/shards/*) and any future mutating routes.
      return container.fetch(request);
    }

    const dataVersion = DATA_VERSIONED.has(url.pathname)
      ? await spireDataVersion(env)
      : null;
    const key = cacheKey(url, env.RELEASE_ID, dataVersion);
    const hit = await fromCache(key);
    if (hit) return hit;

    // A versioned cache miss can be caused by a sync while the container's
    // in-process run cache is still warm. Mark it so the renderer fetches the
    // new data before this response is stored under the new versioned key.
    const originRequest = dataVersion === null ? request : refreshSpireData(request);
    const response = await container.fetch(originRequest);
    if (!response.ok) return response; // 404s stay uncached — cheap and deterministic
    return storeInCache(ctx, key, response);
  },
} satisfies ExportedHandler<Env>;
