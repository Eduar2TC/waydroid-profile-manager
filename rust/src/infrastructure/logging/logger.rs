use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Simple logger that writes to a file and optionally to stdout.
pub struct AppLogger {
    log_file: Option<Mutex<std::fs::File>>,
    buffer: Mutex<Vec<String>>,
}

impl AppLogger {
    pub fn new(log_dir: Option<PathBuf>) -> Self {
        let log_file = log_dir.and_then(|dir| {
            std::fs::create_dir_all(&dir).ok()?;
            let path = dir.join("app.log");
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
                .map(Mutex::new)
        });

        Self {
            log_file,
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub fn log(&self, level: &str, message: &str) {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let hours = (now.as_secs() / 3600) % 24;
        let minutes = (now.as_secs() / 60) % 60;
        let seconds = now.as_secs() % 60;

        let entry = format!(
            "[{:02}:{:02}:{:02}] {:<7} {}",
            hours, minutes, seconds, level, message
        );

        // Always buffer
        if let Ok(mut buf) = self.buffer.lock() {
            buf.push(entry.clone());
            // Keep only last 1000 entries
            if buf.len() > 1000 {
                buf.drain(0..500);
            }
        }

        // Write to file
        if let Ok(mut file) = self.log_file.as_ref().unwrap().lock() {
            let _ = writeln!(file, "{}", entry);
        }
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.buffer.lock().unwrap().clone()
    }

    pub fn clear_logs(&self) {
        self.buffer.lock().unwrap().clear();
    }
}
