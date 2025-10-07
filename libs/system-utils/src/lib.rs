//! System Utilities for hairr OS
//! 
//! Provides common utility functions and helpers for system operations,
//! including logging, timing, error handling, and system information.

use std::time::{SystemTime, UNIX_EPOCH};

/// System time utilities
pub mod time {
    use super::*;

    /// Get current system time in milliseconds since Unix epoch
    pub fn current_time_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Get current system time in microseconds since Unix epoch
    pub fn current_time_us() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    /// Get current system time in nanoseconds since Unix epoch
    pub fn current_time_ns() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    /// Format time as human-readable string
    pub fn format_duration(ms: u64) -> String {
        let seconds = ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;

        if days > 0 {
            format!("{}d {}h {}m", days, hours % 24, minutes % 60)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds % 60)
        } else if seconds > 0 {
            format!("{}s", seconds)
        } else {
            format!("{}ms", ms)
        }
    }
}

/// Memory utilities
pub mod memory {
    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        
        if bytes == 0 {
            return "0 B".to_string();
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Parse memory size string to bytes
    pub fn parse_size(size_str: &str) -> Result<u64, String> {
        let size_str = size_str.trim().to_uppercase();
        let (value, unit) = if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
            let (num, unit) = size_str.split_at(pos);
            (num.trim(), unit.trim())
        } else {
            (size_str.as_str(), "B")
        };

        let value: f64 = value.parse().map_err(|_| "Invalid number")?;

        let multiplier: u64 = match unit {
            "B" => 1,
            "KB" | "K" => 1024,
            "MB" | "M" => 1024 * 1024,
            "GB" | "G" => 1024 * 1024 * 1024,
            "TB" | "T" => 1024u64 * 1024 * 1024 * 1024,
            _ => return Err(format!("Unknown unit: {}", unit)),
        };

        Ok((value * multiplier as f64) as u64)
    }

    /// Align address to specified boundary
    pub fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }

    /// Align address down to specified boundary
    pub fn align_down(addr: usize, align: usize) -> usize {
        addr & !(align - 1)
    }
}

/// String utilities
pub mod string {
    /// Truncate string to specified length with ellipsis
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    /// Pad string to specified length
    pub fn pad_left(s: &str, width: usize, ch: char) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", ch.to_string().repeat(width - s.len()), s)
        }
    }

    /// Pad string to specified length on the right
    pub fn pad_right(s: &str, width: usize, ch: char) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", s, ch.to_string().repeat(width - s.len()))
        }
    }

    /// Convert string to snake_case
    pub fn to_snake_case(s: &str) -> String {
        let mut result = String::new();
        let mut prev_lowercase = false;

        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 && prev_lowercase {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap());
                prev_lowercase = false;
            } else {
                result.push(ch);
                prev_lowercase = ch.is_lowercase();
            }
        }

        result
    }
}

/// Error handling utilities
pub mod error {
    use std::fmt;

    /// System error type
    #[derive(Debug, Clone)]
    pub struct SystemError {
        pub code: u32,
        pub message: String,
        pub component: String,
    }

    impl SystemError {
        pub fn new(code: u32, message: String, component: String) -> Self {
            SystemError {
                code,
                message,
                component,
            }
        }
    }

    impl fmt::Display for SystemError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "[{}] Error {}: {}",
                self.component, self.code, self.message
            )
        }
    }

    impl std::error::Error for SystemError {}

    /// Result type for system operations
    pub type SystemResult<T> = Result<T, SystemError>;
}

/// Logging utilities
pub mod logging {
    use std::sync::Mutex;
    use std::collections::VecDeque;

    /// Log level
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum LogLevel {
        Debug = 0,
        Info = 1,
        Warning = 2,
        Error = 3,
        Critical = 4,
    }

    impl LogLevel {
        pub fn as_str(&self) -> &'static str {
            match self {
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warning => "WARN",
                LogLevel::Error => "ERROR",
                LogLevel::Critical => "CRIT",
            }
        }
    }

    /// Log entry
    #[derive(Debug, Clone)]
    pub struct LogEntry {
        pub timestamp: u64,
        pub level: LogLevel,
        pub component: String,
        pub message: String,
    }

    impl LogEntry {
        pub fn new(level: LogLevel, component: String, message: String) -> Self {
            LogEntry {
                timestamp: crate::time::current_time_ms(),
                level,
                component,
                message,
            }
        }
    }

    /// Simple in-memory logger
    pub struct Logger {
        entries: Mutex<VecDeque<LogEntry>>,
        max_entries: usize,
        min_level: LogLevel,
    }

    impl Logger {
        pub fn new(max_entries: usize, min_level: LogLevel) -> Self {
            Logger {
                entries: Mutex::new(VecDeque::new()),
                max_entries,
                min_level,
            }
        }

        pub fn log(&self, level: LogLevel, component: &str, message: &str) {
            if level < self.min_level {
                return;
            }

            let entry = LogEntry::new(level, component.to_string(), message.to_string());
            let mut entries = self.entries.lock().unwrap();

            if entries.len() >= self.max_entries {
                entries.pop_front();
            }

            entries.push_back(entry.clone());

            // Also print to stdout
            println!("[{}] [{}] {}", level.as_str(), component, message);
        }

        pub fn debug(&self, component: &str, message: &str) {
            self.log(LogLevel::Debug, component, message);
        }

        pub fn info(&self, component: &str, message: &str) {
            self.log(LogLevel::Info, component, message);
        }

        pub fn warning(&self, component: &str, message: &str) {
            self.log(LogLevel::Warning, component, message);
        }

        pub fn error(&self, component: &str, message: &str) {
            self.log(LogLevel::Error, component, message);
        }

        pub fn critical(&self, component: &str, message: &str) {
            self.log(LogLevel::Critical, component, message);
        }

        pub fn get_entries(&self) -> Vec<LogEntry> {
            self.entries.lock().unwrap().iter().cloned().collect()
        }

        pub fn clear(&self) {
            self.entries.lock().unwrap().clear();
        }
    }

    impl Default for Logger {
        fn default() -> Self {
            Logger::new(1000, LogLevel::Info)
        }
    }
}

/// System information utilities
pub mod sysinfo {
    /// System information
    #[derive(Debug, Clone)]
    pub struct SystemInfo {
        pub os_name: String,
        pub os_version: String,
        pub architecture: String,
        pub cpu_count: usize,
        pub hostname: String,
    }

    impl SystemInfo {
        pub fn new() -> Self {
            SystemInfo {
                os_name: "hairr OS".to_string(),
                os_version: "0.1.0".to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                cpu_count: num_cpus::get(),
                hostname: "hairr-system".to_string(),
            }
        }
    }

    impl Default for SystemInfo {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Get system uptime in milliseconds
    pub fn uptime_ms() -> u64 {
        // Simplified - in a real OS, this would read from the kernel
        crate::time::current_time_ms()
    }

    /// Get load average
    pub fn load_average() -> (f32, f32, f32) {
        // Simplified - in a real OS, this would read from the scheduler
        (0.5, 0.7, 0.9)
    }
}

/// Hash utilities
pub mod hash {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    /// Calculate simple hash of data
    pub fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate hash of any hashable type
    pub fn hash_value<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}

/// UUID generation
pub mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Simple UUID v4 generator (not cryptographically secure)
    pub fn generate() -> String {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (time >> 64) as u32,
            ((time >> 48) & 0xFFFF) as u16,
            ((time >> 32) & 0xFFFF) as u16 | 0x4000, // Version 4
            ((time >> 16) & 0xFFFF) as u16 | 0x8000, // Variant
            (time & 0xFFFFFFFFFFFF) as u64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_formatting() {
        assert_eq!(time::format_duration(500), "500ms");
        assert_eq!(time::format_duration(5000), "5s");
        assert_eq!(time::format_duration(65000), "1m 5s");
    }

    #[test]
    fn test_memory_formatting() {
        assert_eq!(memory::format_bytes(0), "0 B");
        assert_eq!(memory::format_bytes(1024), "1.00 KB");
        assert_eq!(memory::format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_memory_parsing() {
        assert_eq!(memory::parse_size("1024").unwrap(), 1024);
        assert_eq!(memory::parse_size("1KB").unwrap(), 1024);
        assert_eq!(memory::parse_size("1MB").unwrap(), 1024 * 1024);
    }

    #[test]
    fn test_memory_alignment() {
        assert_eq!(memory::align_up(100, 16), 112);
        assert_eq!(memory::align_down(100, 16), 96);
    }

    #[test]
    fn test_string_truncate() {
        assert_eq!(string::truncate("hello world", 5), "he...");
        assert_eq!(string::truncate("hello", 10), "hello");
    }

    #[test]
    fn test_string_padding() {
        assert_eq!(string::pad_left("42", 5, '0'), "00042");
        assert_eq!(string::pad_right("42", 5, '0'), "42000");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(string::to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(string::to_snake_case("myTestValue"), "my_test_value");
    }

    #[test]
    fn test_logger() {
        let logger = logging::Logger::new(10, logging::LogLevel::Debug);
        logger.info("test", "Test message");
        logger.error("test", "Error message");
        
        let entries = logger.get_entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_hash() {
        let data = b"hello world";
        let hash1 = hash::hash_bytes(data);
        let hash2 = hash::hash_bytes(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = uuid::generate();
        let uuid2 = uuid::generate();
        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.len(), 36); // Standard UUID format
    }
}
