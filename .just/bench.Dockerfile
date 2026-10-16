# Linux x86_64 with Valgrind, gungraun-runner and jq, to count instructions.
FROM ubuntu:24.04

ARG RUST_TOOLCHAIN
ARG GUNGRAUN_VERSION

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential ca-certificates git jq libc6-dbg rustup valgrind \
    && rm -rf /var/lib/apt/lists/*
# rustfmt and clippy match the toolchain file, so rustup does not fetch them on every run.
RUN rustup toolchain install "$RUST_TOOLCHAIN" --profile minimal --component rustfmt,clippy --no-self-update
ENV RUSTUP_TOOLCHAIN="$RUST_TOOLCHAIN" PATH="/root/.cargo/bin:$PATH"
RUN cargo install --locked --version "$GUNGRAUN_VERSION" gungraun-runner
RUN git config --system --add safe.directory '*'
