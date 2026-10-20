# Validation and platform status

This record describes the development snapshot checked on **2026-10-17**. It is not a guarantee that every future commit or platform combination has been tested. Run the commands in [TESTING.md](TESTING.md) for the checkout you are using.

## Platform status

| Platform | Recorded status |
|---|---|
| macOS ARM64 | Release build, automated suite, lints and app-bundle verification passed. Interactive use and visual feedback were reported during development. |
| Windows | Successful native compilation, automated testing and interactive use were reported by a user. This report is not a separate automated CI result. |
| Linux | Intended target; compilation, native-window behavior and installation packages have not yet been validated. |

The recorded toolchain was Rust 1.99.1 with egui/eframe 0.33.3. The repository includes `Cargo.lock` to preserve the resolved dependency versions.

## Recorded automated checks

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo test --locked --workspace` | 98 passed: 17 engine/storage/model tests, 74 function-catalog tests, 5 egui UI tests and 2 icon-asset tests |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Passed |
| `cargo build --locked --release` | Passed on macOS ARM64 |
| `sh scripts/package-macos.sh` | Created an application bundle containing the native release executable |
| `codesign --verify --deep --strict` on the macOS bundle | Passed with a local ad-hoc signature |
| `plutil -lint` on the bundle's Info.plist | Passed |

The validation runs used cached dependencies; `--offline` can be added to the Cargo commands after the appropriate platform dependencies are available.

## What the tests establish

The function catalog includes named checks for every implemented scientific function and additional checks for programmer operations, base/width/signedness combinations, unit conversion, RPN, memory and history restoration.

The egui checks render headless frames in all main modes, plus RPN and conversion configurations. They cover keypad workflows, keyboard entry/repeated Enter in the expression field, alignment of conversion controls at several widths, visible keypad bounds, and excessive empty space below the keypad. They do not create OS windows.

Storage tests verify serialization and replacement of files in disposable temporary directories. They do not use an existing application history file.

See [TESTING.md](TESTING.md) and [the function catalog](core/tests/function_catalog.rs) for the cases and coverage limits.

## Remaining verification

- Linux builds, graphics integration, X11/Wayland behavior and distribution packages.
- Repeatable native-window smoke checks on each platform. One restricted development environment could not access macOS UI services, so that attempt did not establish native rendering; the headless checks and reported interactive use are separate evidence.
- Screen-reader behavior and other platform integrations beyond the existing tests.
- Public distribution signing/notarization. The macOS bundle script currently provides ad-hoc signing only.
- Numerical equivalence to Apple's Calculator; the application uses its own documented arithmetic policies.

## Windows renderer update — 2026-10-20

A fresh Windows 11 Enterprise VM and Store review reported startup failure. The diagnostic build reported `egui_glow requires OpenGL 2.0+`, identifying OpenGL initialization as the immediate failure.

Windows now selects wgpu with the Direct3D 12 backend. Adapter selection prefers compatible hardware and retains CPU/software adapters, including Windows WARP, as a fallback. macOS and Linux explicitly retain the Glow renderer. Startup diagnostics report the configured renderer.

On macOS ARM64, formatting, Clippy with warnings denied, the release build, and all 99 tests passed. The additional test compiles the Windows graphics configuration on the host and checks Direct3D selection and the no-adapter error path. It does not initialize Direct3D or establish successful Windows startup. Native Windows compilation, standalone execution in the affected VM, software-adapter rendering, and Store-package activation still require verification.

Currency and Math Notes are not implemented and are not part of the test coverage.
