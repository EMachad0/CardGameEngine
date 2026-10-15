# Benchmark comparisons. Recipes run from the repo root.
set working-directory := '..'

[doc("Bench a git ref and the working tree on this machine, and print a markdown report")]
compare base:
    #!/usr/bin/env bash
    set -euo pipefail
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
    scratch="$(mktemp -d)"
    trap 'git worktree remove --force "$scratch/base" 2>/dev/null || true; rm -rf "$scratch"' EXIT
    git worktree add --quiet --detach "$scratch/base" "{{base}}"
    export CRITERION_HOME="$scratch/criterion"
    bench() {
        local out="$1"
        shift
        cargo bench --workspace --bench '*' -- --color never "$@" 2>&1 | tee "$out" >&2
    }
    touch "$scratch/base.txt"
    if compgen -G "$scratch/base/crates/*/benches/*.rs" > /dev/null; then
        (cd "$scratch/base" && bench "$scratch/base.txt" --save-baseline base)
    fi
    bench "$scratch/head.txt" --baseline-lenient base
    awk -f .just/bench-report.awk "$scratch/base.txt" "$scratch/head.txt"
