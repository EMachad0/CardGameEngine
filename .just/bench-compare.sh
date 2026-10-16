#!/usr/bin/env bash
# Counts instructions for every `*_counts` bench at a base ref and in the working tree, then prints
# a markdown report. Needs Linux, valgrind, gungraun-runner and jq. Runs from the repo root.
set -euo pipefail

base_ref="$1"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
scratch="$CARGO_TARGET_DIR/bench-compare"
cleanup() {
    git worktree remove --force "$scratch/base" 2> /dev/null || true
    rm -rf "$scratch"
}
cleanup
trap cleanup EXIT
mkdir -p "$scratch"
git worktree add --quiet --detach "$scratch/base" "$base_ref"
export GUNGRAUN_HOME="$scratch/gungraun"

has_counts() {
    cargo metadata --no-deps --format-version 1 \
        | jq -e '[.packages[].targets[] | select(.name | endswith("_counts"))] | length > 0' \
            > /dev/null
}
counts() {
    cargo bench --workspace --bench '*_counts' -- --output-format=json > "$1"
}

: > "$scratch/base.json"
if (cd "$scratch/base" && has_counts); then
    (cd "$scratch/base" && counts "$scratch/base.json")
fi
counts "$scratch/head.json"
jq -n -r --slurpfile base "$scratch/base.json" --slurpfile head "$scratch/head.json" \
    -f .just/bench-report.jq
