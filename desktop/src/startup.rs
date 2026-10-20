use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Diagnostics {
    path: Option<PathBuf>,
}

impl Diagnostics {
    pub fn install() -> Self {
        let preferred = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("SoftOxideCalculator"));
        let fallback = std::env::temp_dir().join("SoftOxideCalculator");
        let path = preferred
            .into_iter()
            .chain(std::iter::once(fallback))
            .find_map(|directory| {
                fs::create_dir_all(&directory).ok()?;
                let path = directory.join("startup.log");
                // Start a fresh log for each launch so this diagnostic cannot grow indefinitely.
                fs::write(&path, "").ok()?;
                Some(path)
            });
        let diagnostics = Self { path };
        diagnostics.record(&format!(
            "Starting SoftOxide Calculator {} on {} / {}; renderer=Glow (OpenGL)",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
        ));
        diagnostics.record(&format!("Executable: {:?}", std::env::current_exe()));
        let panic_diagnostics = diagnostics.clone();
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            panic_diagnostics.report(&format!(
                "Application panic: {info}\n{}",
                std::backtrace::Backtrace::force_capture()
            ));
            previous_hook(info);
        }));
        diagnostics
    }

    pub fn record(&self, message: &str) {
        if let Some(path) = &self.path {
            if let Ok(mut file) = OpenOptions::new().append(true).open(path) {
                let _ = writeln!(file, "{message}");
            }
        }
    }

    pub fn report(&self, message: &str) {
        self.record(message);
        let location = match &self.path {
            Some(path) => format!("Diagnostic log: {}", path.display()),
            None => "The diagnostic log could not be written.".into(),
        };
        let text = format!("SoftOxide Calculator could not continue.\n\n{message}\n\n{location}");
        eprintln!("{text}");
        #[cfg(target_os = "windows")]
        show_error_dialog(&text);
    }
}

#[cfg(target_os = "windows")]
fn show_error_dialog(message: &str) {
    use std::ffi::c_void;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(owner: *mut c_void, text: *const u16, title: *const u16, flags: u32) -> i32;
    }

    let text: Vec<u16> = message
        .replace('\0', " ")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let title: Vec<u16> = "SoftOxide Calculator — startup error"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: both strings are NUL-terminated and remain valid during the synchronous call.
    // A null owner is allowed; MB_ICONERROR | MB_OK needs no other pointers.
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
    }
}
