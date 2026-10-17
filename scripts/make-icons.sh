#!/bin/sh
# Rebuild platform assets from the original master on macOS.
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ "$(uname -s)" != Darwin ]; then
    echo "Icon regeneration uses macOS sips. Prebuilt assets are committed for other platforms." >&2
    exit 1
fi
build_dir="$project_dir/target/icon-build"
mkdir -p "$build_dir/png"
for size in 16 24 32 48 64 128 256 512 1024; do
    sips -z "$size" "$size" "$project_dir/assets/icon-master.png" --out "$build_dir/png/$size.png" >/dev/null
done
cp "$build_dir/png/512.png" "$project_dir/assets/icon.png"
rustc --edition=2021 "$project_dir/tools/write-icons.rs" -o "$build_dir/write-icons"
"$build_dir/write-icons" "$build_dir/png" "$project_dir/assets/icon.ico" "$project_dir/assets/icon.icns"
echo "Built assets/icon.png, assets/icon.icns and assets/icon.ico"
