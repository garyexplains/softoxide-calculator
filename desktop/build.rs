//! Embed the committed ICO in native MSVC executables, using the Windows SDK.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn sdk_resource_compiler(bin: &Path, architecture: &str) -> Option<PathBuf> {
    let direct = bin.join(architecture).join("rc.exe");
    if direct.is_file() {
        return Some(direct);
    }
    let mut versions: Vec<_> = fs::read_dir(bin)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .collect();
    versions.sort();
    versions
        .into_iter()
        .rev()
        .map(|version| version.join(architecture).join("rc.exe"))
        .find(|path| path.is_file())
}
fn resource_compiler() -> PathBuf {
    if let Some(path) = env::var_os("RC") {
        return path.into();
    }
    let host = env::var("HOST").unwrap_or_default();
    let architecture = if host.starts_with("aarch64") {
        "arm64"
    } else if host.starts_with("i686") {
        "x86"
    } else {
        "x64"
    };
    if let Some(paths) = env::var_os("PATH") {
        if let Some(rc) = env::split_paths(&paths)
            .map(|dir| dir.join("rc.exe"))
            .find(|path| path.is_file())
        {
            return rc;
        }
    }
    for key in ["WindowsSdkVerBinPath", "WindowsSdkBinPath"] {
        if let Some(bin) = env::var_os(key) {
            if let Some(rc) = sdk_resource_compiler(Path::new(&bin), architecture) {
                return rc;
            }
        }
    }
    for key in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(root) = env::var_os(key) {
            let bin = Path::new(&root).join("Windows Kits/10/bin");
            if let Some(rc) = sdk_resource_compiler(&bin, architecture) {
                return rc;
            }
        }
    }
    panic!("Windows SDK resource compiler rc.exe was not found. Install the Windows SDK with the C++ build tools, use a Developer PowerShell, or set RC to rc.exe's full path.");
}
fn main() {
    println!("cargo:rerun-if-changed=../assets/icon.ico");
    for key in [
        "RC",
        "WindowsSdkVerBinPath",
        "WindowsSdkBinPath",
        "ProgramFiles(x86)",
        "ProgramFiles",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("windows-msvc") {
        return;
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let icon = root
        .join("../assets/icon.ico")
        .canonicalize()
        .expect("Missing assets/icon.ico");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let source = out.join("softoxide-icon.rc");
    let resource = out.join("softoxide-icon.res");
    let icon_path = icon
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    fs::write(&source, format!("1 ICON \"{icon_path}\"\n"))
        .expect("Cannot write icon resource source");
    let result = Command::new(resource_compiler())
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&source)
        .output()
        .expect("Cannot launch Windows SDK resource compiler");
    assert!(
        result.status.success(),
        "Windows icon resource compilation failed: {} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    println!("cargo:rustc-link-arg-bins={}", resource.display());
}
