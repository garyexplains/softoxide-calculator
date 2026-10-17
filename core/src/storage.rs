//! Versioned UTF-8 history/settings storage. No access to Apple's calculator data.
use crate::{programmer::IntegerConfig, Angle, Calculator, HistoryEntry, Mode};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CALCULATOR_DATA_DIR") {
        return Some(PathBuf::from(path).join("session-v1.txt"));
    }
    let root = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
    };
    root.map(|p| p.join("SoftOxideCalculator/session-v1.txt"))
}
fn escape(text: &str) -> String {
    text.replace('%', "%25")
        .replace('\t', "%09")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}
fn unescape(text: &str) -> String {
    text.replace("%0D", "\r")
        .replace("%0A", "\n")
        .replace("%09", "\t")
        .replace("%25", "%")
}
fn mode_number(mode: Mode) -> u8 {
    match mode {
        Mode::Basic => 0,
        Mode::Scientific => 1,
        Mode::Programmer => 2,
    }
}
fn mode(text: &str) -> io::Result<Mode> {
    match text {
        "0" => Ok(Mode::Basic),
        "1" => Ok(Mode::Scientific),
        "2" => Ok(Mode::Programmer),
        _ => Err(invalid()),
    }
}
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Invalid calculator session file; original file was preserved",
    )
}
fn flag(text: &str) -> io::Result<bool> {
    match text {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(invalid()),
    }
}
type Settings = (Mode, Angle, IntegerConfig, bool, Option<(usize, usize)>);
fn config(fields: &[&str]) -> io::Result<Settings> {
    if fields.len() != 9 {
        return Err(invalid());
    }
    let m = mode(fields[0])?;
    let angle = if flag(fields[1])? {
        Angle::Degrees
    } else {
        Angle::Radians
    };
    let integer = IntegerConfig {
        base: fields[2].parse().map_err(|_| invalid())?,
        width: fields[3].parse().map_err(|_| invalid())?,
        signed: flag(fields[4])?,
        arithmetic_shift: flag(fields[5])?,
    };
    integer.validate().map_err(|_| invalid())?;
    let rpn = flag(fields[6])?;
    let conversion = if fields[7] == "-" && fields[8] == "-" {
        None
    } else {
        let a: usize = fields[7].parse().map_err(|_| invalid())?;
        let b: usize = fields[8].parse().map_err(|_| invalid())?;
        if a >= crate::units::UNITS.len()
            || b >= crate::units::UNITS.len()
            || crate::units::UNITS[a].category != crate::units::UNITS[b].category
        {
            return Err(invalid());
        }
        Some((a, b))
    };
    if (m == Mode::Programmer && (rpn || conversion.is_some())) || (rpn && conversion.is_some()) {
        return Err(invalid());
    }
    Ok((m, angle, integer, rpn, conversion))
}
fn config_text(
    m: Mode,
    a: Angle,
    i: IntegerConfig,
    rpn: bool,
    conversion: Option<(usize, usize)>,
) -> String {
    let (from, to) = conversion
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .unwrap_or(("-".into(), "-".into()));
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        mode_number(m),
        a == Angle::Degrees,
        i.base,
        i.width,
        i.signed,
        i.arithmetic_shift,
        rpn,
        from,
        to
    )
}
pub fn encode(c: &Calculator) -> String {
    let mut text = format!(
        "softoxide-calculator-v1\n{}\n",
        config_text(c.mode, c.angle, c.integer, c.rpn, c.conversion)
    );
    for h in &c.history {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            h.timestamp,
            escape(&h.expression),
            escape(&h.result),
            config_text(h.mode, h.angle, h.integer, h.rpn, h.conversion)
        ));
    }
    text
}
pub fn decode(text: &str) -> io::Result<Calculator> {
    if text.len() > 2_000_000 {
        return Err(invalid());
    }
    let mut lines = text.lines();
    // Accept the original file header so existing users retain their history.
    if !matches!(
        lines.next(),
        Some("softoxide-calculator-v1" | "rust-calculator-v1")
    ) {
        return Err(invalid());
    }
    let fields: Vec<_> = lines.next().ok_or_else(invalid)?.split('\t').collect();
    let (mode, angle, integer, rpn, conversion) = config(&fields)?;
    let mut calc = Calculator::default();
    calc.mode = mode;
    calc.angle = angle;
    calc.integer = integer;
    calc.rpn = rpn;
    calc.conversion = conversion;
    for line in lines {
        let f: Vec<_> = line.split('\t').collect();
        if f.len() != 12 {
            return Err(invalid());
        }
        let (mode, angle, integer, rpn, conversion) = config(&f[3..])?;
        let timestamp = f[0].parse().map_err(|_| invalid())?;
        calc.history.push(HistoryEntry {
            timestamp,
            expression: unescape(f[1]),
            result: unescape(f[2]),
            mode,
            angle,
            integer,
            rpn,
            conversion,
        });
        if calc.history.len() > 200 {
            return Err(invalid());
        }
    }
    Ok(calc)
}
pub fn load(path: &Path) -> io::Result<Calculator> {
    load_session(path, std::env::var_os("CALCULATOR_DATA_DIR").is_none())
}
fn load_session(path: &Path, import_legacy: bool) -> io::Result<Calculator> {
    match fs::read_to_string(path) {
        Ok(text) => decode(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // Import the previous application's data without changing its file.
            // Explicit CALCULATOR_DATA_DIR overrides stay isolated.
            if import_legacy {
                if let Some(parent) = path
                    .parent()
                    .filter(|p| p.file_name() == Some(std::ffi::OsStr::new("SoftOxideCalculator")))
                {
                    if let Some(root) = parent.parent() {
                        let legacy = root.join("RustCalculator/session-v1.txt");
                        match fs::read_to_string(legacy) {
                            Ok(text) => return decode(&text),
                            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                            Err(err) => return Err(err),
                        }
                    }
                }
            }
            Ok(Calculator::default())
        }
        Err(e) => Err(e),
    }
}
pub fn save(path: &Path, c: &Calculator) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temp, encode(c))?;
    #[cfg(target_os = "windows")]
    if path.exists() {
        let backup = path.with_extension("previous");
        fs::rename(path, &backup)?;
        if let Err(err) = fs::rename(&temp, path) {
            let _ = fs::rename(&backup, path);
            return Err(err);
        }
        let _ = fs::remove_file(backup);
        return Ok(());
    }
    fs::rename(temp, path)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_roundtrip_and_corruption() {
        let mut c = Calculator::default();
        c.input = "200+10%".into();
        c.equals();
        c.mode = Mode::Scientific;
        c.angle = Angle::Radians;
        let text = encode(&c);
        let d = decode(&text).unwrap();
        assert_eq!(d.mode, c.mode);
        assert_eq!(d.angle, c.angle);
        assert_eq!(d.history[0].expression, "200+10%");
        assert_eq!(d.history[0].result, "220");
        let legacy = text.replacen("softoxide-calculator-v1", "rust-calculator-v1", 1);
        assert_eq!(decode(&legacy).unwrap().history[0].result, "220");
        assert!(decode("junk").is_err());
        assert!(decode(&text.replace("false", "maybe")).is_err());
    }
    #[test]
    fn atomic_save_reload() {
        let root = std::env::temp_dir().join(format!(
            "softoxide-calculator-storage-test-{}",
            std::process::id()
        ));
        let path = root.join("session.txt");
        let mut c = Calculator::default();
        c.input = "1+2".into();
        c.equals();
        save(&path, &c).unwrap();
        c.input = "4+5".into();
        c.edited();
        c.equals();
        save(&path, &c).unwrap();
        assert_eq!(load(&path).unwrap().history.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn legacy_history_import_preserves_original_and_prefers_new_file() {
        let root = std::env::temp_dir().join(format!(
            "softoxide-calculator-migration-test-{}",
            std::process::id()
        ));
        let path = root.join("SoftOxideCalculator/session-v1.txt");
        let old_path = root.join("RustCalculator/session-v1.txt");
        let mut c = Calculator::default();
        c.input = "2+3".into();
        c.equals();
        let old_text = encode(&c).replacen("softoxide-calculator-v1", "rust-calculator-v1", 1);
        fs::create_dir_all(old_path.parent().unwrap()).unwrap();
        fs::write(&old_path, &old_text).unwrap();
        assert!(load_session(&path, false).unwrap().history.is_empty());
        let imported = load_session(&path, true).unwrap();
        assert_eq!(imported.history[0].result, "5");
        assert_eq!(fs::read_to_string(&old_path).unwrap(), old_text);
        save(&path, &imported).unwrap();
        assert!(fs::read_to_string(&path)
            .unwrap()
            .starts_with("softoxide-calculator-v1\n"));
        fs::write(&old_path, "invalid old history").unwrap();
        assert_eq!(load_session(&path, true).unwrap().history.len(), 1);
        fs::write(&path, "invalid new history").unwrap();
        assert!(load_session(&path, true).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
