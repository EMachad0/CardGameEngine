mod wk ".just/wk.just"
mod check ".just/check.just"
mod bench ".just/bench.just"

# List all recipes, including those in modules.
default:
    @just --list --list-submodules
