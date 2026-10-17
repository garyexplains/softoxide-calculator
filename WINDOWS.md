# Build and test on Windows

These instructions build a native Windows application from this GitHub repository. If a prebuilt Windows download is available under **Releases**, you can use it without installing Rust or the build tools below.

## Prerequisites

Install a current stable Rust toolchain using the [official Windows installer](https://rust-lang.org/tools/install/). Accept the native **MSVC** toolchain for your processor architecture.

Rustup may offer to install Microsoft's prerequisites. If installing them separately, select **Desktop development with C++** in Visual Studio Installer / Build Tools, including the matching MSVC tools and Windows SDK. The full Visual Studio IDE is not required. See the [Rust MSVC setup guide](https://rust-lang.github.io/rustup/installation/windows-msvc.html).

Use native Windows PowerShell or Terminal for a Windows executable. WSL builds a Linux executable. For an Intel/AMD PC use the x64 tools; for ARM Windows use the corresponding ARM64 tools.

Open a new terminal after installation and verify:

```powershell
rustc --version
cargo --version
rustc -vV
```

The `host` line should end in `pc-windows-msvc`.

## Get the source

Install Git if necessary. Copy the clone URL from this repository's **Code** menu, then replace `REPOSITORY_URL` in the following commands:

```powershell
git clone REPOSITORY_URL softoxide-calculator
cd softoxide-calculator
```

Alternatively, choose **Code → Download ZIP** on GitHub, extract it, and open PowerShell in the extracted repository directory. It must contain `Cargo.toml`, `Cargo.lock`, `core`, `desktop` and `scripts`.

Run all remaining commands from that repository root. Cargo downloads the dependencies needed for Windows on the first build; internet access is required. Build artifacts and local dependency caches from other operating systems are not needed.

## Test

```powershell
cargo test --locked --workspace
```

The suite tests calculations and the headless egui interface without manual keypad input or changes to your saved application history. See [TESTING.md](TESTING.md) for coverage details.

To select tests by name:

```powershell
cargo test --locked --workspace scientific_
cargo test --locked --workspace programmer_
cargo test --locked --workspace conversion_
```

The optional wrapper `powershell -File scripts/test.ps1` runs the suite too. Direct Cargo commands work without changing PowerShell's script execution policy.

## Build and run

```powershell
cargo build --locked --release
.\target\release\softoxide-calculator.exe
```

Or build and launch with one command:

```powershell
cargo run --locked --release
```

The result is a native egui window. The default history/settings file is `%APPDATA%\SoftOxideCalculator\session-v1.txt`.

For an isolated manual trial, use a separate terminal and set a different data directory before launching:

```powershell
$env:CALCULATOR_DATA_DIR = Join-Path $env:TEMP 'SoftOxideCalculatorTrial'
.\target\release\softoxide-calculator.exe
```

Close that terminal when finished, or remove its override with `Remove-Item Env:CALCULATOR_DATA_DIR`. The trial data remains in the chosen directory until you remove it.

## Formatting and lint checks

```powershell
rustup component add clippy rustfmt
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
```

## Update a checkout

If your working tree is clean, fetch the latest code and rerun the checks:

```powershell
git pull --ff-only
cargo test --locked --workspace
cargo build --locked --release
```

Close the running calculator before rebuilding its executable. If you are building a particular release, check out that release's tag before building.

## Troubleshooting

- **`cargo` is not found:** open a new terminal and confirm the Rust installation completed.
- **`link.exe` or Windows SDK errors:** install the C++ workload and matching SDK, then reopen the terminal. If needed, use the Visual Studio Developer PowerShell / Native Tools prompt for your architecture.
- **Dependency-download errors:** allow access to crates.io and its crate downloads. Use `--offline` only after the required Windows dependencies are cached.
- **Tests pass but the application cannot open a window:** check graphics drivers and OpenGL availability. Headless tests do not create the native graphics context.

Successful Windows compilation, testing and use have been reported. See [VALIDATION.md](VALIDATION.md) for the recorded platform status. When reporting a failure, include the command, complete error output, Windows version and `rustc -vV` output.
