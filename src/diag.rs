use crate::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG: Mutex<Option<File>> = Mutex::new(None);

pub fn file_name() -> &'static str {
    if cfg!(debug_assertions) {
        "debug.log"
    } else {
        "gkms_fl.log"
    }
}

pub fn init() -> Result<(), Error> {
    let dir = crate::config::exe_dir().ok_or(Error::LogWriteFailed)?;
    let path = dir.join(file_name());
    let file = File::create(&path).map_err(|_| Error::LogWriteFailed)?;
    let mut slot = LOG.lock().unwrap_or_else(|e| e.into_inner());
    *slot = Some(file);
    drop(slot);
    info(&format!(
        "gkms_fl {} {} os={} arch={}",
        env!("CARGO_PKG_VERSION"),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        std::env::consts::OS,
        std::env::consts::ARCH,
    ));
    if let Ok(exe) = std::env::current_exe() {
        info(&format!("exe={}", exe.display()));
    }
    info(&format!("log={}", path.display()));
    Ok(())
}

pub fn info(msg: &str) {
    write_line("INFO", msg);
}

pub fn error(msg: &str) {
    write_line("ERROR", msg);
}

pub fn debug(msg: &str) {
    if cfg!(debug_assertions) {
        write_line("DEBUG", msg);
    }
}

fn write_line(level: &str, msg: &str) {
    let mut slot = LOG.lock().unwrap_or_else(|e| e.into_inner());
    let Some(file) = slot.as_mut() else {
        return;
    };
    let _ = writeln!(file, "{} [{level}] {msg}", timestamp());
    let _ = file.flush();
}

fn timestamp() -> String {
    let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return "0".into();
    };
    let secs = dur.as_secs();
    let ms = dur.subsec_millis();
    let days = (secs / 86400) as i64;
    let tod = secs % 86400;
    let (y, mo, d) = civil_utc(days);
    format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}.{ms:03}Z",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

fn civil_utc(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}
