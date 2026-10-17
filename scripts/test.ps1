$ErrorActionPreference = 'Stop'
$projectDir = Split-Path -Parent $PSScriptRoot
$cache = Join-Path $projectDir '.cargo-cache'
if (-not $env:CARGO_HOME -and (Test-Path (Join-Path $cache 'registry/src'))) {
    $env:CARGO_HOME = $cache
}
& cargo test --locked --manifest-path (Join-Path $projectDir 'Cargo.toml') --workspace @args
exit $LASTEXITCODE
