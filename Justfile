default:
    @just --list

# Enable the repository-managed Git hooks in this checkout
install-hooks:
    proto install lefthook
    lefthook install

# Seed local fitness data, start its Worker API, then run Topcoat with live reload
dev port="3000":
    bash scripts/dev.sh "{{port}}"

# Build the debug binary and extract its assets
build:
    cargo build
    topcoat asset bundle --bin benjisponge

# Build the release binary and extract its assets
release:
    cargo build --release
    topcoat asset bundle --release --bin benjisponge

# Build the container image, sync its bundled assets, and deploy to Cloudflare
deploy:
    docker build -f deploy/Dockerfile -t benjisponge-build .
    docker rm -f benjisponge-extract 2>/dev/null || true
    docker create --name benjisponge-extract benjisponge-build
    rm -rf deploy/assets/_topcoat && mkdir -p deploy/assets/_topcoat
    docker cp benjisponge-extract:/app/assets deploy/assets/_topcoat/assets
    docker rm benjisponge-extract
    rm -f deploy/assets/_topcoat/assets/manifest.toml
    cd deploy && npx wrangler deploy --var RELEASE_ID:$(git rev-parse --short HEAD)

# Upload new Slay the Spire 2 runs to the site's database (see --help)
sync-spire *args:
    cargo run --bin spire_sync -- {{args}}

# Upload a Strong workout CSV export to the site's fitness database (see --help)
sync-fitness csv *args:
    cargo run --bin fitness_sync -- "{{csv}}" {{args}}

# Run formatting, lint, and test checks
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test

# Run the test suite
test:
    cargo test
