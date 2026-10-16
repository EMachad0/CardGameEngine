# Instruction count comparisons. Recipes run from the repo root.
set working-directory := '..'

[doc("Count instructions at a git ref and in the working tree, and print a markdown report (Linux with valgrind, gungraun-runner and jq)")]
compare base:
    bash .just/bench-compare.sh "{{base}}"

[doc("Run compare in a Linux x86_64 container, the platform CI counts on")]
docker_compare base:
    #!/usr/bin/env bash
    set -euo pipefail
    toolchain="$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)"
    gungraun="$(cargo pkgid gungraun | sed 's/.*@//')"
    image="card-game-bench:$toolchain-gungraun-$gungraun"
    docker build --quiet --platform linux/amd64 \
        --build-arg RUST_TOOLCHAIN="$toolchain" --build-arg GUNGRAUN_VERSION="$gungraun" \
        -t "$image" -f .just/bench.Dockerfile .just > /dev/null
    common="$(cd "$(git rev-parse --git-common-dir)" && pwd)"
    docker run --rm --platform linux/amd64 --security-opt seccomp=unconfined \
        -v "$PWD:$PWD" -v "$common:$common" -w "$PWD" \
        -v card-game-bench-target:/bench/target -e CARGO_TARGET_DIR=/bench/target \
        -v card-game-bench-registry:/root/.cargo/registry \
        "$image" bash .just/bench-compare.sh "{{base}}"
