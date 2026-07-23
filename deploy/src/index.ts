import { Container, getContainer } from "@cloudflare/containers";
import { cacheKey, cacheable, fromCache, storeInCache } from "./cache";
import { handleFitness } from "./fitness";

// Secrets are not part of the generated Env type; handlers extend it locally
// (same pattern as fitness.ts). POSTGRES_URL and the sync tokens exist so the
// constructor below can forward them into the container process.
type ShellEnv = Env & {
  POSTGRES_URL?: string;
  SPIRE_SYNC_TOKEN?: string;
  FITNESS_SYNC_TOKEN?: string;
};

export class BenjispongeContainer extends Container<ShellEnv> {
  defaultPort = 8080;
  sleepAfter = "15m";

  constructor(ctx: BenjispongeContainer["ctx"], env: ShellEnv) {
    super(ctx, env);
    // envVars is the only channel into the container process, and it is read
    // at instance start — rotating a secret needs a container restart.
    // Unset secrets become empty strings, which the Rust side treats as
    // "closed" (auth) or "unconfigured" (database).
    this.envVars = {
      SITE_ORIGIN: "https://benjisponge.com",
      POSTGRES_URL: env.POSTGRES_URL ?? "",
      SPIRE_SYNC_TOKEN: env.SPIRE_SYNC_TOKEN ?? "",
      FITNESS_SYNC_TOKEN: env.FITNESS_SYNC_TOKEN ?? "",
    };
  }
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

    // Public fitness reads and the private bounded CSV import live in the
    // shared site D1 database and never touch the container. (The spire API
    // moved into the container app — Rust + Postgres; fitness follows in the
    // next migration phase.)
    if (url.pathname.startsWith("/api/fitness")) {
      return handleFitness(request, env, url);
    }

    const container = getContainer(env.SITE_CONTAINER, "site");

    // API responses are never edge-cached, whatever their method — the
    // container marks them no-store, but don't even consult the cache.
    if (url.pathname.startsWith("/api/") || !cacheable(request)) {
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
