#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ -z "${CARGO_HOME:-}" ] && [ -d "$project_dir/.cargo-cache/registry/src" ]; then
    export CARGO_HOME="$project_dir/.cargo-cache"
fi
exec cargo test --locked --manifest-path "$project_dir/Cargo.toml" --workspace "$@"
