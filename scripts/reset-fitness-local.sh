#!/usr/bin/env bash

set -Eeuo pipefail

workout_csv="${1:-/home/benji/Downloads/WorkoutData.csv}"
fitness_api="http://127.0.0.1:8791"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

if [[ ! -r "${workout_csv}" ]]; then
    printf 'reset-fitness-local: CSV is not readable: %s\n' "${workout_csv}" >&2
    exit 1
fi

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

printf 'reset-fitness-local: replacing the six local fitness tables\n'
(
    cd "${repo_root}/deploy"
    npx wrangler d1 execute SITE_DB --local --yes --command \
        "DROP TABLE IF EXISTS set_records;
         DROP TABLE IF EXISTS sets;
         DROP TABLE IF EXISTS exercise_tags;
         DROP TABLE IF EXISTS exercises;
         DROP TABLE IF EXISTS workouts;
         DROP TABLE IF EXISTS fitness_meta;"
    npx wrangler d1 execute SITE_DB --local --yes --file=fitness-schema.sql
)

printf 'reset-fitness-local: importing %s\n' "${workout_csv}"
cd "${repo_root}"
FITNESS_SYNC_TOKEN=local-development cargo run --bin fitness_sync -- \
    "${workout_csv}" \
    --api "${fitness_api}" \
    --json
