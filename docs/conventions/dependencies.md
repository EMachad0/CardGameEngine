# Dependencies

## When to use

Read this before adding, upgrading or removing a dependency, or changing the features a crate
enables.

## Ask first

The user decides every new dependency, every upgrade and every newly enabled feature. That covers
dev-dependencies, and a dependency that serves one call site. Ask before writing it, and name:

- the crate and its version
- the member crates that will use it
- why std or an existing workspace dependency falls short

Removals go through normal review.

If a dependency clears the bar in `docs/adr/README.md`, as an ECS would, it gets a record.

## Versions

Take the version from crates.io, never from memory. `cargo info <crate>` prints the latest one.

## Workspace dependencies

The root `Cargo.toml` holds the version, under `[workspace.dependencies]`, and no member crate
repeats it. A member writes `<crate> = { workspace = true }` and lists the features it uses on that
line. A member can't turn default features off, so `default-features = false` goes in the
workspace entry.

Write the workspace entry first, by hand. `cargo add -p <member> <crate>` then writes
`{ workspace = true }`. Without the entry, it writes a version into the member.

## Lock files

Never edit `Cargo.lock` or `.mise/mise.lock` by hand. Cargo and mise write them. To move a locked
crate version, run `cargo update -p <crate>`.
