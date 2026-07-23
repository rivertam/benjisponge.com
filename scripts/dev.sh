#!/usr/bin/env bash

set -Eeuo pipefail

site_port="${1:-3000}"
fitness_port=8791
fitness_api="http://127.0.0.1:${fitness_port}"
pg_container=benjisponge-pg
pg_port=5490
pg_url="postgresql://postgres:dev@127.0.0.1:${pg_port}/benjisponge"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
wrangler_pid=""

cleanup() {
    local status=$?
    trap - EXIT INT TERM HUP
    if [[ -n "${wrangler_pid}" ]]; then
        kill -TERM -- "-${wrangler_pid}" 2>/dev/null || true
        wait "${wrangler_pid}" 2>/dev/null || true
    fi
    # The Postgres container stays up between dev sessions (named volume
    # benjisponge-pg-data holds the data); `docker rm -f benjisponge-pg`
    # to reclaim it.
    exit "${status}"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

# Local Postgres (18, matching production) for the spire data the site now
# reads in-process. Seed runs with:
#   just sync-spire --api "http://127.0.0.1:${site_port}"
printf 'dev: ensuring Postgres on 127.0.0.1:%s\n' "${pg_port}"
if [[ "$(docker inspect -f '{{.State.Running}}' "${pg_container}" 2>/dev/null)" != "true" ]]; then
    if docker inspect "${pg_container}" >/dev/null 2>&1; then
        docker start "${pg_container}" >/dev/null
    else
        docker run -d --name "${pg_container}" \
            -e POSTGRES_PASSWORD=dev -e POSTGRES_DB=benjisponge \
            -v benjisponge-pg-data:/var/lib/postgresql \
            -p "127.0.0.1:${pg_port}:5432" \
            postgres:18-alpine >/dev/null
    fi
fi
until docker exec "${pg_container}" pg_isready -U postgres -q 2>/dev/null; do
    sleep 0.3
done

printf 'dev: applying migrations\n'
(cd "${repo_root}" && POSTGRES_URL="${pg_url}" cargo run --quiet --bin migrate -- migration apply)

printf 'dev: starting fitness API on %s\n' "${fitness_api}"
(
    cd "${repo_root}/deploy"
    exec setsid npx wrangler dev \
        --local \
        --ip 127.0.0.1 \
        --port "${fitness_port}" \
        --var FITNESS_SYNC_TOKEN:local-development \
        --log-level warn \
        --show-interactive-dev-session=false
) &
wrangler_pid=$!

printf 'dev: starting Topcoat on port %s\n' "${site_port}"
cd "${repo_root}"
FITNESS_DATA_ORIGIN="${fitness_api}" \
    POSTGRES_URL="${pg_url}" \
    SPIRE_SYNC_TOKEN=local-development \
    PORT="${site_port}" topcoat dev --bin benjisponge
