# SoftOxide Calculator

A native desktop calculator inspired by Apple Calculator for macOS, written in Rust with [egui/eframe](https://github.com/emilk/egui). It combines basic, scientific, programmer and reverse Polish notation (RPN) modes in one application, with offline unit conversion and calculation history.

## Features

- Basic arithmetic, parentheses, percentages, repeated equals and memory.
- Scientific functions: powers, roots, logarithms, exponentials, factorial, circular and hyperbolic trigonometry, and their inverses. Supports degrees and radians.
- Programmer mode: exact fixed-width integers, binary/octal/decimal/hexadecimal input, bitwise operations, shifts, rotates, byte/word flips, and a bit display.
- RPN mode with Enter, Drop, Swap and stack rotation.
- Offline conversions between 27 units across length, mass, temperature, time, volume and area.
- Editable expressions, keyboard input, clipboard copying, always-on-top, and persistent calculation history.

## Get the app

Prebuilt applications, when available, are attached to this repository's **Releases**. Choose an asset matching your operating system and processor architecture. Prebuilt applications do not require Rust.

GitHub's automatically generated **Source code** archives contain source files, not a compiled application. If a suitable binary has not been published, follow the build instructions below.

## Build from source

Install a current stable [Rust toolchain](https://rust-lang.org/tools/install/) and the native tools for your platform:

| Platform | Prerequisites |
|---|---|
| Windows | MSVC toolchain, Microsoft C++ build tools and Windows SDK. See [Windows setup](WINDOWS.md). |
| macOS | Xcode Command Line Tools; install them with `xcode-select --install` if needed. |
| Linux | C/C++ build tools, `pkg-config`, and development libraries for X11/Wayland, OpenGL and xkbcommon. See the upstream [egui Linux prerequisites](https://github.com/emilk/egui/tree/0.33.3#demo). |

Copy this repository's clone URL from GitHub's **Code** menu. Replace `REPOSITORY_URL` below with that URL:

```sh
git clone REPOSITORY_URL softoxide-calculator
cd softoxide-calculator
cargo run --locked --release
```

You can also use GitHub's **Code → Download ZIP**, extract the source, and open a terminal in the extracted directory containing `Cargo.toml`. All commands in this documentation assume that directory is the current working directory.

The first build needs internet access to download dependencies. `Cargo.lock` pins the dependency versions. Build on the destination operating system to produce its native executable:

```sh
cargo build --locked --release
```

| Platform | Build output |
|---|---|
| Windows | `target\release\softoxide-calculator.exe` |
| macOS / Linux | `target/release/softoxide-calculator` |

For detailed Windows installation, testing and troubleshooting, see [WINDOWS.md](WINDOWS.md).

### Create a macOS app bundle

On macOS, the included script builds a release executable and packages it as an application:

```sh
sh scripts/package-macos.sh
open "target/SoftOxide Calculator.app"
```

The generated bundle is ad-hoc signed for local use. Developer ID signing and notarization are separate distribution steps. The script runs on macOS only.

## Using the calculator

Enter values with the keypad, or click the expression field to type or paste a formula.

| Input | Action |
|---|---|
| Enter / Return | Evaluate; push an entry in RPN mode |
| Escape | Clear the expression and stack |
| Backspace / Del button | Delete the last character |
| C | Clear the current operand |
| AC | Clear calculation state; preserve memory |
| MC / MR / M+ / M− | Clear, recall, add to or subtract from memory |
| 2nd | Show inverse scientific functions |

Outside the expression editor, arithmetic keys use the keypad actions, `p` inserts pi, and the system copy shortcut copies the result. Inside the expression editor, normal text selection and editing apply. The display's Copy button copies the result directly.

Examples:

| Expression | Mode | Result |
|---|---|---|
| `2 + 3 * 4` | Basic | `14` |
| `200 + 10%` | Basic | `220` |
| `sin(30)` | Scientific, degrees | `0.5` |
| `root(-8,3)` | Scientific | `-2` |
| `log(8,2)` | Scientific | `3` |
| `FFFFFFFFFFFFFFFF+1` | Programmer, 64-bit hex | `0` |

Scientific functions apply to the last operand. Two-argument root/log buttons start a function expression: enter the second argument and closing `)`. In RPN, those operations consume the top two stack values directly. Enter duplicates the top RPN value when no entry is pending.

`^` means power in real-number modes and XOR in programmer mode. `%` means percent in real-number modes and remainder in programmer mode.

Changing the main mode clears the current expression/stack and disables RPN and conversion. Changing the programmer input base preserves the bit pattern. Calculation history retains up to 200 records; select an entry to restore its calculation context. Memory, the active expression and the RPN stack are not preserved between application sessions.

## Numerical behavior

Real-number calculations use IEEE 754 binary64 (`f64`). Display rounding reduces representation noise but does not provide arbitrary precision or exact decimal arithmetic. Use Programmer mode for exact integers beyond binary64's precision.

- Powers associate right to left; unary minus binds below power.
- Percentage following addition/subtraction is relative to the left operand.
- Factorial accepts nonnegative integers through 170.
- Complex results, infinities and invalid domains are reported as errors.
- Circular trig follows the angle setting; hyperbolic functions do not convert angles.
- Random values are in `[0, 1)` and use a noncryptographic generator.
- Programmer arithmetic wraps at 8, 16, 32 or 64 bits. Signed mode controls division, remainder and decimal display; arithmetic right shift is independently selectable.
- Shift counts at or above the word width produce zero for logical shifts or sign extension for arithmetic right shifts. Rotations move one bit.
- Byte flip exchanges adjacent byte pairs; word flip exchanges adjacent 16-bit pairs within 32-bit groups.

This is an independent implementation inspired by desktop calculators. Exact behavior matching Apple's Calculator has not been established. Currency rates, Math Notes, locale-specific number parsing, arbitrary precision, and platform-specific widgets/Touch Bar integrations are not implemented.

## Data storage

The application keeps its own history and settings in `session-v1.txt`:

| Platform | Default location |
|---|---|
| Windows | `%APPDATA%\SoftOxideCalculator\session-v1.txt` |
| macOS | `~/Library/Application Support/SoftOxideCalculator/session-v1.txt` |
| Linux | `$XDG_DATA_HOME/SoftOxideCalculator/session-v1.txt`, or `~/.local/share/SoftOxideCalculator/session-v1.txt` when unset |

Existing history from earlier development builds is imported when the new data file does not yet exist; the previous file is preserved. Explicit `CALCULATOR_DATA_DIR` overrides do not import other data.

Set `CALCULATOR_DATA_DIR` to use another directory. Settings and history save after changes. If the file is malformed, the application reports the error and disables saving for that session to preserve the file. The app does not read another calculator's history.

## Development and testing

Run the automated suite without clicking through the keypad:

```sh
cargo test --locked --workspace
```

For formatting and lint checks:

```sh
rustup component add rustfmt clippy
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
```

See [TESTING.md](TESTING.md) for function-specific tests, test coverage and native smoke checks. [VALIDATION.md](VALIDATION.md) records platform validation and its limits. Linux support still needs build/runtime validation; macOS has automated build/test coverage, and successful Windows compilation, testing and use have been reported.

## Application icon

Platform-ready PNG, ICO and ICNS assets are included under [assets/](assets/README.md). The desktop app embeds its window icon; the Windows MSVC build and macOS bundle script also attach the platform icon to their executables/bundles. Linux packaging can use the included desktop-entry template and PNG.

## Project structure

| Path | Responsibility |
|---|---|
| [core/src/expression.rs](core/src/expression.rs) | Real-number parser, evaluation and formatting |
| [core/src/programmer.rs](core/src/programmer.rs) | Fixed-width integer operations |
| [core/src/model.rs](core/src/model.rs) | Entry editing, memory, repeated equals, RPN and history |
| [core/src/units.rs](core/src/units.rs) | Offline unit conversion |
| [core/src/storage.rs](core/src/storage.rs) | Versioned settings/history persistence |
| [core/tests/function_catalog.rs](core/tests/function_catalog.rs) | Named function regression tests |
| [desktop/src/app.rs](desktop/src/app.rs) | Native egui interface |
| [desktop/src/main.rs](desktop/src/main.rs) | Desktop application entry point |
| [scripts/](scripts/) | Test runners and macOS packaging |
