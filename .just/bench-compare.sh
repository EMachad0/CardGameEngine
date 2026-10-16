#!/usr/bin/env bash
# Counts instructions for every `*_counts` bench at a base ref and in the working tree, then prints
# a markdown report. Needs Linux, valgrind, gungraun-runner and jq. Runs from the repo root.
set -euo pipefail

base_ref="$1"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
scratch="$CARGO_TARGET_DIR/bench-compare"
rm -rf "$scratch"
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/base"
# An export, not a worktree, so the run leaves the repository's git state alone.
git archive "$base_ref" | tar -x -C "$scratch/base"
export GUNGRAUN_HOME="$scratch/gungraun"

has_counts() {
    cargo metadata --no-deps --format-version 1 \
        | jq -e '[.packages[].targets[] | select(.name | endswith("_counts"))] | length > 0' \
            > /dev/null
}
gungraun_version() {
    cargo pkgid gungraun 2> /dev/null | sed 's/.*@//'
}
counts() {
    cargo bench --workspace --bench '*_counts' -- --output-format=json > "$1"
}

# gungraun-runner has to match the gungraun library, and it is installed for the head's.
head_version="$(gungraun_version)"
base_version="$(cd "$scratch/base" && gungraun_version)"
note=""
: > "$scratch/base.json"
if [ -n "$base_version" ] && (cd "$scratch/base" && has_counts); then
    if [ "$base_version" = "$head_version" ]; then
        (cd "$scratch/base" && counts "$scratch/base.json")
    else
        note="The base uses gungraun $base_version and the head $head_version, so the base isn't counted."
    fi
fi
counts "$scratch/head.json"
if [ -n "$note" ]; then
    printf '%s\n\n' "$note"
fi
jq -n -r --slurpfile base "$scratch/base.json" --slurpfile head "$scratch/head.json" \
    -f .just/bench-report.jq
