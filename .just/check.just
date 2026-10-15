# Quality gates for the Rust workspace. Recipes run from the repo root.
set working-directory := '..'

[doc("Format check, import paths, lint, and test")]
all: fmt paths lint test

[doc("Check formatting")]
fmt:
    cargo fmt --all --check

[doc("Check that each definition is imported by one path")]
paths:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=0
    report() {
        if [ -n "$2" ]; then
            printf '%s\n%s\n\n' "$1" "$2"
            failed=1
        fi
    }
    word='(^|[^[:alnum:]_])'

    report "Only rules::prelude may name the prelude; import from the defining module instead:" \
        "$(grep -rnw prelude crates/rules/src \
            | grep -v -e '^crates/rules/src/prelude\.rs:' \
                -e '^crates/rules/src/lib\.rs:[0-9]*:pub mod prelude;$')"

    report "Import from rules::prelude by name, not with a glob:" \
        "$(find crates -name '*.rs' -print | sort | while read -r file; do
            tr -s '[:space:]' ' ' < "$file" \
                | grep -Eq "${word}rules::([^;]*[^[:alnum:]_])?prelude::(\*|\{[^}]*\*)" \
                && echo "$file"
        done)"

    report "Spell paths from crate::, not super:: or self:: (use super::*; in a test module is fine):" \
        "$(grep -rnE "${word}(super|self)::" crates/*/src \
            | grep -vE '^[^:]+:[0-9]+:[[:space:]]*use super::\*;[[:space:]]*$')"

    report "Re-export only in prelude.rs:" \
        "$(grep -rnE "${word}pub(\([^)]*\))?[[:space:]]+use([[:space:]]|$)" crates/*/src \
            | grep -v '^crates/[^/]*/src/prelude\.rs:')"

    exit "$failed"

[doc("Lint with clippy (warnings are errors)")]
lint:
    cargo clippy --workspace --all-targets -- -D warnings

[doc("Run tests")]
test:
    cargo test --workspace
