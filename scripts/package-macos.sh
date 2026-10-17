#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ "$(uname -s)" != Darwin ]; then
    echo "Run this script on macOS." >&2
    exit 1
fi
if [ -z "${CARGO_HOME:-}" ] && [ -d "$project_dir/.cargo-cache/registry/src" ]; then
    export CARGO_HOME="$project_dir/.cargo-cache"
fi
cargo build --locked --release --manifest-path "$project_dir/Cargo.toml"
bundle_dir="$project_dir/target/SoftOxide Calculator.app"
mkdir -p "$bundle_dir/Contents/MacOS"
cp "$project_dir/target/release/softoxide-calculator" "$bundle_dir/Contents/MacOS/SoftOxideCalculator"
cat > "$bundle_dir/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>SoftOxideCalculator</string>
  <key>CFBundleIdentifier</key><string>com.softoxide.calculator</string>
  <key>CFBundleName</key><string>SoftOxide Calculator</string>
  <key>CFBundleDisplayName</key><string>SoftOxide Calculator</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict></plist>
PLIST
codesign --force --sign - "$bundle_dir"
echo "$bundle_dir"
