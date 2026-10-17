# Application icon

The icon uses a graphite calculator face and warm orange operation keys. Pixels outside the rounded tile are transparent.

| Asset | Use |
|---|---|
| [icon-master.png](icon-master.png) | Original full-resolution artwork |
| [icon.png](icon.png) | 512 × 512 PNG embedded in the egui window; Linux packaging source |
| [icon.ico](icon.ico) | Windows icon with 16, 24, 32, 48, 64, 128 and 256px images |
| [icon.icns](icon.icns) | macOS icon with standard and Retina representations through 1024px |
| [com.softoxide.calculator.desktop](com.softoxide.calculator.desktop) | Desktop-entry template for Linux packaging |
| [icon-prompt.txt](icon-prompt.txt) | Artwork generation specification |

The desktop app embeds the PNG, so users do not need external asset files next to the executable. Native Windows MSVC builds embed the ICO into the executable using the Windows SDK resource compiler (`rc.exe`). The macOS packaging script copies the ICNS into the app's Resources folder and declares it in Info.plist.

Linux packages should install the desktop-entry template to their applications directory and the PNG as `com.softoxide.calculator.png` in a 512 × 512 hicolor application-icon directory. The desktop entry expects `softoxide-calculator` to be on PATH. The template is provided for packaging; running Cargo does not install it into the system.

To regenerate platform assets from the master on macOS:

```sh
sh scripts/make-icons.sh
```

The script uses `sips` for resizing and [tools/write-icons.rs](../tools/write-icons.rs) to package PNGs into ICO/ICNS containers. It writes intermediates under the ignored `target/icon-build` directory. The generated platform assets are committed so Windows/Linux contributors do not need macOS to build the app.

For native Windows icon compilation, install the Windows SDK alongside the MSVC C++ tools. The build script searches the toolchain PATH and Windows SDK directories. `RC` can override the resource compiler's location when needed.
