use std::{
    io::Write,
    sync::{Mutex, OnceLock},
};

/// Global log file handle, initialized once per process.
static LOG_FILE: OnceLock<Mutex<Option<std::fs::File>>> = OnceLock::new();

/// Maximum log file size before rotation (5 MB).
const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;

/// Initialize file logging to the given directory.
/// Creates `harmonium.log` (and rotates to `harmonium.log.old` when full).
/// Safe to call multiple times — only the first call takes effect.
pub fn init_file_logging(log_dir: &std::path::Path) {
    LOG_FILE.get_or_init(|| {
        let log_path = log_dir.join("harmonium.log");

        // Rotate if existing log is too large
        if let Ok(meta) = std::fs::metadata(&log_path) {
            if meta.len() > MAX_LOG_SIZE {
                let old = log_dir.join("harmonium.log.old");
                let _ = std::fs::rename(&log_path, &old);
            }
        }

        match std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            Ok(file) => {
                eprintln!("Logging to {}", log_path.display());
                Mutex::new(Some(file))
            }
            Err(e) => {
                eprintln!("WARN: could not open log file {}: {}", log_path.display(), e);
                Mutex::new(None)
            }
        }
    });
}

/// Write a formatted line to the log file (if initialized).
fn write_to_file(level: &str, msg: &str) {
    if let Some(mutex) = LOG_FILE.get() {
        if let Ok(mut guard) = mutex.lock() {
            if let Some(file) = &mut *guard {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                // Simple ISO-ish timestamp from epoch seconds
                let _ = writeln!(file, "[{now}] {level}: {msg}");
                let _ = file.flush();
            }
        }
    }
}

pub fn info(msg: &str) {
    write_to_file("INFO", msg);
    if std::env::var("HARMONIUM_CLI").is_err() {
        eprintln!("{msg}");
    }
}

pub fn warn(msg: &str) {
    write_to_file("WARN", msg);
    if std::env::var("HARMONIUM_CLI").is_err() {
        eprintln!("WARN: {msg}");
    }
}

pub fn error(msg: &str) {
    write_to_file("ERROR", msg);
    // Always show errors even in CLI mode
    eprintln!("ERROR: {msg}");
}
