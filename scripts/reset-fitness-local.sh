#!/usr/bin/env bash

set -Eeuo pipefail

workout_csv="${1:-/home/benji/Downloads/WorkoutData.csv}"
fitness_api="${FITNESS_API:-http://127.0.0.1:3000}"
pg_container=benjisponge-pg
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

if [[ ! -r "${workout_csv}" ]]; then
    printf 'reset-fitness-local: CSV is not readable: %s\n' "${workout_csv}" >&2
    exit 1
fi

# Prove the local site is up and the local token works before destroying
# anything: an authorized-but-empty import must come back 400.
auth_status="$(
    curl --silent --output /dev/null --write-out '%{http_code}' \
        --request POST \
        --header 'Authorization: Bearer local-development' \
        --header 'Content-Type: application/json' \
        --data '{}' \
        "${fitness_api}/api/fitness/import" || true
)"
if [[ "${auth_status}" != 400 ]]; then
    printf 'reset-fitness-local: run `just dev` first (local API auth probe returned %s)\n' \
        "${auth_status}" >&2
    exit 1
fi

printf 'reset-fitness-local: truncating the local fitness tables\n'
docker exec "${pg_container}" psql -U postgres -d benjisponge -q \
    -c "TRUNCATE TABLE sets, exercise_tags, exercises, workouts;" \
    -c "UPDATE fitness_meta SET v = 0 WHERE k = 'version';"

printf 'reset-fitness-local: importing %s\n' "${workout_csv}"
cd "${repo_root}"
FITNESS_SYNC_TOKEN=local-development cargo run --bin fitness_sync -- \
    "${workout_csv}" \
    --api "${fitness_api}" \
    --json
