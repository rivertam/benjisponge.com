#!/usr/bin/env bash

set -Eeuo pipefail

site_port="${1:-3000}"
fitness_port=8791
fitness_api="http://127.0.0.1:${fitness_port}"
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
    exit "${status}"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

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
FITNESS_DATA_ORIGIN="${fitness_api}" PORT="${site_port}" topcoat dev --bin benjisponge
