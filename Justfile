default:
    @just --list

# Start the development server with live reload
dev port="3000":
    PORT={{port}} topcoat dev

# Build the debug binary and extract its assets
build:
    cargo build
    topcoat asset bundle

# Build the release binary and extract its assets
release:
    cargo build --release
    topcoat asset bundle --release

# Run formatting and lint checks
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# Capture visual snapshots, optionally with a label
snapshot label="":
    scripts/snapshot "{{label}}"

# Compare two snapshot directories
snapshot-diff before after:
    scripts/snapshot-diff "{{before}}" "{{after}}"
