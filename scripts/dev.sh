#!/usr/bin/env bash

set -Eeuo pipefail

site_port="${1:-3000}"
fitness_port=8791
fitness_api="http://127.0.0.1:${fitness_port}"
workout_csv="${WORKOUT_DATA_CSV:-/home/benji/Downloads/WorkoutData.csv}"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
deploy_dir="${repo_root}/deploy"
wrangler_pid=""
dev_env_file=""

port_is_open() {
    (exec 3<>"/dev/tcp/127.0.0.1/${1}") 2>/dev/null
}

stop_wrangler() {
    if [[ -z "${wrangler_pid}" ]]; then
        return
    fi

    local wrangler_target=""
    if kill -0 -- "-${wrangler_pid}" 2>/dev/null; then
        wrangler_target="-${wrangler_pid}"
    elif kill -0 "${wrangler_pid}" 2>/dev/null; then
        # Covers an interrupt between spawning the subshell and setsid
        # establishing the Wrangler process group.
        wrangler_target="${wrangler_pid}"
    fi
    if [[ -n "${wrangler_target}" ]]; then
        printf '\ndev: stopping fitness API\n'
        kill -TERM -- "${wrangler_target}" 2>/dev/null || true
        for _attempt in {1..50}; do
            if ! kill -0 -- "${wrangler_target}" 2>/dev/null; then
                break
            fi
            sleep 0.1
        done
        if kill -0 -- "${wrangler_target}" 2>/dev/null; then
            kill -KILL -- "${wrangler_target}" 2>/dev/null || true
        fi
    fi
    wait "${wrangler_pid}" 2>/dev/null || true
    wrangler_pid=""
}

cleanup() {
    local status=$?
    trap - EXIT INT TERM HUP
    stop_wrangler
    if [[ -n "${dev_env_file}" ]]; then
        rm -f -- "${dev_env_file}"
    fi
    exit "${status}"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

if [[ ! "${site_port}" =~ ^[0-9]+$ ]]; then
    printf 'dev: invalid Topcoat port: %s\n' "${site_port}" >&2
    exit 2
fi
site_port_number=$((10#${site_port}))
if ((site_port_number < 1 || site_port_number > 65535)); then
    printf 'dev: invalid Topcoat port: %s\n' "${site_port}" >&2
    exit 2
fi
site_port="${site_port_number}"
if ((site_port == fitness_port)); then
    printf 'dev: Topcoat port %s is reserved for the local fitness API\n' "${fitness_port}" >&2
    exit 2
fi
if [[ ! -r "${workout_csv}" ]]; then
    printf 'dev: workout CSV is not readable: %s\n' "${workout_csv}" >&2
    printf 'dev: set WORKOUT_DATA_CSV to use a different export\n' >&2
    exit 1
fi
if port_is_open "${fitness_port}"; then
    printf 'dev: port %s is already in use; refusing to start a second fitness API\n' \
        "${fitness_port}" >&2
    exit 1
fi
if port_is_open "${site_port}"; then
    printf 'dev: Topcoat port %s is already in use\n' "${site_port}" >&2
    exit 1
fi

printf 'dev: applying local fitness schema\n'
(
    cd "${deploy_dir}"
    npx wrangler d1 execute SITE_DB --local --file=fitness-schema.sql --yes
)

dev_env_file="$(mktemp "${TMPDIR:-/tmp}/benjisponge-fitness.env.XXXXXX")"
chmod 600 "${dev_env_file}"
dev_token="$(openssl rand -hex 32)"
printf 'FITNESS_SYNC_TOKEN=%s\n' "${dev_token}" >"${dev_env_file}"

printf 'dev: starting fitness API on %s\n' "${fitness_api}"
(
    cd "${deploy_dir}"
    exec setsid npx wrangler dev \
        --local \
        --ip 127.0.0.1 \
        --port "${fitness_port}" \
        --env-file "${dev_env_file}" \
        --log-level warn \
        --show-interactive-dev-session=false
) &
wrangler_pid=$!

fitness_ready=false
for _attempt in {1..300}; do
    if ! kill -0 "${wrangler_pid}" 2>/dev/null; then
        wait "${wrangler_pid}" || true
        printf 'dev: fitness API exited before becoming ready\n' >&2
        exit 1
    fi
    if curl --fail --silent --max-time 1 \
        "${fitness_api}/api/fitness/ids" >/dev/null 2>&1; then
        fitness_ready=true
        break
    fi
    sleep 0.1
done
if [[ "${fitness_ready}" != true ]]; then
    printf 'dev: fitness API did not become ready within 30 seconds\n' >&2
    exit 1
fi

printf 'dev: syncing %s into local D1\n' "${workout_csv}"
(
    cd "${repo_root}"
    FITNESS_SYNC_TOKEN="${dev_token}" cargo run --bin fitness_sync -- \
        "${workout_csv}" \
        --api "${fitness_api}" \
        --json
)

printf 'dev: fitness API ready at %s; starting Topcoat on port %s\n' \
    "${fitness_api}" "${site_port}"
cd "${repo_root}"
PORT="${site_port}" topcoat dev --bin benjisponge
