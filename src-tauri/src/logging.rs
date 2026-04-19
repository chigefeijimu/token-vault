// Logging module for tracking application runtime status and debugging
// Provides structured logging with timestamps, levels, and context

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub context: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, module: &str, message: &str, context: Option<&str>) -> Self {
        Self {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            level,
            module: module.to_string(),
            message: message.to_string(),
            context: context.map(|s| s.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        let ctx = self.context.as_ref()
            .map(|c| format!(" [{}]", c))
            .unwrap_or_default();
        format!(
            "[{}] [{}] [{}]{} {}",
            self.timestamp, self.level, self.module, ctx, self.message
        )
    }
}

/// Global logging state
static LOG_FILE: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));
static MIN_LEVEL: Lazy<Mutex<LogLevel>> = Lazy::new(|| Mutex::new(LogLevel::Info));

/// Initialize logging with optional file output
pub fn init(log_level: Option<&str>, log_file: Option<PathBuf>) {
    // Set minimum log level
    if let Some(level_str) = log_level {
        let level = LogLevel::from_str(level_str);
        *MIN_LEVEL.lock().unwrap() = level;
    }

    // Set log file path
    if let Some(path) = log_file {
        *LOG_FILE.lock().unwrap() = Some(path);
    }

    info!("logging", "Logging system initialized");
}

/// Set minimum log level at runtime
pub fn set_level(level: LogLevel) {
    *MIN_LEVEL.lock().unwrap() = level;
}

/// Get current log level
pub fn get_level() -> LogLevel {
    *MIN_LEVEL.lock().unwrap()
}

/// Write log entry to file
fn write_to_file(entry: &LogEntry) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref path) = *guard {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(file, "{}", entry.to_string());
            }
        }
    }
}

/// Core logging function
fn log(level: LogLevel, module: &str, message: &str, context: Option<&str>) {
    let min_level = *MIN_LEVEL.lock().unwrap();
    
    if level < min_level {
        return;
    }

    let entry = LogEntry::new(level, module, message, context);
    
    // Print to stderr for debug builds, stdout for release
    #[cfg(debug_assertions)]
    {
        eprintln!("{}", entry.to_string());
    }
    #[cfg(not(debug_assertions))]
    {
        println!("{}", entry.to_string());
    }

    // Write to file if configured
    write_to_file(&entry);
}

/// Log a trace message
#[macro_export]
macro_rules! log_trace {
    ($module:expr, $message:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Trace, $module, &msg, None);
    };
    ($module:expr, $message:expr, $context:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Trace, $module, &msg, Some($context));
    };
}

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($module:expr, $message:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Debug, $module, &msg, None);
    };
    ($module:expr, $message:expr, $context:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Debug, $module, &msg, Some($context));
    };
}

/// Log an info message
#[macro_export]
macro_rules! log_info {
    ($module:expr, $message:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Info, $module, &msg, None);
    };
    ($module:expr, $message:expr, $context:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Info, $module, &msg, Some($context));
    };
}

/// Log a warning message
#[macro_export]
macro_rules! log_warn {
    ($module:expr, $message:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Warn, $module, &msg, None);
    };
    ($module:expr, $message:expr, $context:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Warn, $module, &msg, Some($context));
    };
}

/// Log an error message
#[macro_export]
macro_rules! log_error {
    ($module:expr, $message:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Error, $module, &msg, None);
    };
    ($module:expr, $message:expr, $context:expr $(, $arg:expr)*) => {
        let msg = format!($message, $($arg),*);
        crate::logging::log(crate::logging::LogLevel::Error, $module, &msg, Some($context));
    };
}

/// Log with error context (automatically includes error details)
#[macro_export]
macro_rules! log_error_with {
    ($module:expr, $message:expr, $error:expr $(, $arg:expr)*) => {
        let err_msg = format!("{:?}", $error);
        let full_msg = format!("{} | Error: {}", format!($message, $($arg),*), err_msg);
        crate::logging::log(crate::logging::LogLevel::Error, $module, &full_msg, None);
    };
}

/// Get all recent log entries from file
pub fn get_recent_logs(limit: usize) -> Result<Vec<LogEntry>, String> {
    if let Ok(guard) = LOG_FILE.lock() {
        if let Some(ref path) = *guard {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read log file: {}", e))?;
            
            let entries: Vec<LogEntry> = content
                .lines()
                .rev()
                .take(limit)
                .filter_map(|line| parse_log_line(line))
                .collect();
            
            return Ok(entries);
        }
    }
    Ok(vec![])
}

/// Parse a log line back into a LogEntry
fn parse_log_line(line: &str) -> Option<LogEntry> {
    // Expected format: [timestamp] [level] [module] [context] message
    // Or: [timestamp] [level] [module] message
    
    let parts: Vec<&str> = line.splitn(5, ']').collect();
    if parts.len() < 4 {
        return None;
    }
    
    let timestamp = parts[0].trim_start_matches('[').to_string();
    let level_str = parts[1].trim().trim_start_matches('[');
    let level = LogLevel::from_str(level_str);
    let module = parts[2].trim().trim_start_matches('[');
    
    let rest = parts.get(3..).map(|p| p.join("]"));
    
    let (context, message) = if let Some(r) = rest {
        if let Some((ctx, msg)) = r.split_once("] ") {
            let ctx_clean = ctx.trim_start_matches('[');
            if !ctx_clean.is_empty() && !ctx_clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':') {
                (None, r.trim_start_matches("] "))
            } else {
                (Some(ctx_clean.to_string()), msg.to_string())
            }
        } else {
            (None, r.trim_start_matches("] "))
        }
    } else {
        (None, String::new())
    };
    
    Some(LogEntry {
        timestamp,
        level,
        module: module.to_string(),
        message,
        context,
    })
}

/// Clear the log file
pub fn clear_logs() -> Result<(), String> {
    if let Ok(guard) = LOG_FILE.lock() {
        if let Some(ref path) = *guard {
            File::create(path)
                .map_err(|e| format!("Failed to clear log file: {}", e))?;
        }
    }
    Ok(())
}

/// Get log file path
pub fn get_log_file_path() -> Option<String> {
    LOG_FILE.lock().ok()?.as_ref().map(|p| p.to_string_lossy().to_string())
}