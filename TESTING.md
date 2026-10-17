# Testing

Run these commands from the repository root, the directory containing `Cargo.toml`. The same Cargo commands work on Windows, macOS and Linux after installing the platform's build prerequisites.

## Run the complete suite

```sh
cargo test --locked --workspace
```

The current suite contains **96 named tests**, including 74 function-catalog tests. Some tests exercise matrices with many individual inputs. Cargo prints `ok` or `FAILED` for each named test and reports a summary for each test binary. Failures produce a nonzero exit code and assertion details.

The automated suite does not open OS windows or modify your application history. Persistence tests use disposable directories under the operating system's temporary directory. The first run needs internet access to fetch build dependencies; subsequent runs can use `--offline` when all required dependencies are cached.

Optional wrapper scripts run the same suite:

```sh
# macOS / Linux
sh scripts/test.sh
```

```powershell
# Windows PowerShell
powershell -File scripts/test.ps1
```

## Select tests

Filter by a test-name fragment:

```sh
cargo test --locked --workspace scientific_
cargo test --locked --workspace programmer_
cargo test --locked --workspace conversion_
cargo test --locked --workspace rpn_
cargo test --locked --workspace memory_
```

To list names without running tests:

```sh
cargo test --locked --workspace -- --list
```

To test only the portable engine package:

```sh
cargo test --locked -p calculator-core
```

## Coverage

| Area | Automated checks |
|---|---|
| Basic | Arithmetic, precedence, parentheses, scientific notation entry, contextual percentages, clear/delete/sign, repeated equals |
| Scientific | Each implemented trig/inverse/hyperbolic function, angle conversion, powers/roots, logs, exponentials, reciprocal, factorial, constants, random range, invalid domains and overflow |
| Programmer | Arithmetic, bitwise operations, shifts/rotates, complements, flips, exact large integers; base × width × signedness round trips and boundary conditions |
| Conversion | Reference conversions, compatible pairs among the 27 units with round trips, incompatible dimensions and absolute zero |
| RPN | Entry/duplication, operand order, two-argument scientific functions, unary operations, percent, drop/swap/rotation and transactional errors |
| Memory/history | Memory actions in every main mode, history restoration, duplicate handling, serialization and file replacement |
| egui | Headless rendering, keypad workflows, focused expression input, conversion-control alignment, visible keypad bounds and excess space below the keypad |

Known-answer tests compare results against expected values. Invariant tests check properties such as converting a quantity out and back, or formatting/parsing an exact integer. They exercise the same Rust engine and state model as the desktop app. UI tests invoke egui and application handlers; they do not click every widget in a native OS window.

## Add a regression test

The function catalog is [core/tests/function_catalog.rs](core/tests/function_catalog.rs). Add a named calculation case for another expected answer:

```rust
calculation!(scientific_sin_90, "sin(90)", 1.0);
```

Use `Calculator` for workflows involving editing, modes, memory, history or the stack. Native UI tests live in [desktop/src/app.rs](desktop/src/app.rs) and can inject egui input or inspect layout bounds.

Run the suite after adding the case. Test-only changes do not require rebuilding or reopening an installed application.

## Native-window smoke check

This optional check launches the compiled application, renders several frames and exits. It needs a graphical desktop and working graphics drivers. It is separate from the headless suite.

Build first:

```sh
cargo build --locked --release
```

On macOS/Linux:

```sh
CALCULATOR_DATA_DIR="$(mktemp -d)" CALCULATOR_SMOKE_TEST=1 ./target/release/softoxide-calculator
```

On Windows, in a separate PowerShell terminal:

```powershell
$env:CALCULATOR_DATA_DIR = Join-Path $env:TEMP ('SoftOxideCalculatorSmoke-' + [guid]::NewGuid())
$env:CALCULATOR_SMOKE_TEST = '1'
.\target\release\softoxide-calculator.exe
```

Close that terminal afterwards to discard its environment overrides. A successful smoke check reaches the rendered-frame marker and exits. It does not prove that every feature, clipboard action or platform integration works.

## Manual verification

Check appearance, mathematical symbols, window resizing, clipboard interaction and assistive technology on each supported platform. Exercise X11 and Wayland separately on Linux where available. Automated headless checks cannot establish native-window appearance or graphics-driver compatibility.

The tests cover documented calculator behavior, not numerical equivalence to Apple's private engine or every possible input. See [VALIDATION.md](VALIDATION.md) for recorded validation results.

## Formatting and lints

```sh
rustup component add rustfmt clippy
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
```
