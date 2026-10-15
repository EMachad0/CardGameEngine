# Benchmark comparisons. Recipes run from the repo root.
set working-directory := '..'

[doc("Bench a git ref and the working tree on this machine, and print a critcmp table")]
compare base:
    #!/usr/bin/env bash
    set -euo pipefail
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
    scratch="$(mktemp -d)"
    trap 'git worktree remove --force "$scratch/base" 2>/dev/null || true; rm -rf "$scratch"' EXIT
    git worktree add --quiet --detach "$scratch/base" "{{base}}"
    export CRITERION_HOME="$scratch/criterion"
    bench() {
        cargo bench --workspace --bench '*' -- --save-baseline "$1" >&2
    }
    if compgen -G "$scratch/base/crates/*/benches/*.rs" > /dev/null; then
        (cd "$scratch/base" && bench base)
    fi
    bench head
    mise exec -- critcmp --target-dir "$scratch" base head
