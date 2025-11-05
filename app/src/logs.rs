use LogLevel::*;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "display_hat"))]
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cmp::Ordering, fmt, str::FromStr};

#[cfg(feature = "display_hat")]
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    TRACE,
    DEBUG,
    #[default]
    INFO,
    WARN,
    ERROR,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TRACE => write!(f, "TRACE"),
            DEBUG => write!(f, "DEBUG"),
            INFO => write!(f, "INFO"),
            WARN => write!(f, "WARN"),
            ERROR => write!(f, "ERROR"),
        }
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        let rank = |level: &LogLevel| match level {
            ERROR => 5,
            WARN => 4,
            INFO => 3,
            DEBUG => 2,
            TRACE => 1,
        };
        rank(self).cmp(&rank(other))
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "debug" => Ok(LogLevel::DEBUG),
            "trace" => Ok(LogLevel::TRACE),
            "info" => Ok(LogLevel::INFO),
            "warn" => Ok(LogLevel::WARN),
            "error" => Ok(LogLevel::ERROR),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub fields: Option<Fields>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Fields {
    pub message: String,
    pub tip: Option<String>,
}

#[cfg(not(feature = "display_hat"))]
fn random_index(n: u64, max: usize) -> usize {
    n as usize % max
}

#[cfg(not(feature = "display_hat"))]
fn random_log_entry() -> LogEntry {
    const LEVELS: [LogLevel; 5] = [ERROR, WARN, INFO, DEBUG, TRACE];
    const MESSAGES: [&str; 10] = [
        "Initializing system",
        "Connection established",
        "User login succeeded",
        "File not found",
        "Database timeout",
        "Processing request",
        "Cache miss",
        "Low disk space",
        "Reconnected to server",
        "Shutdown complete",
    ];

    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let level = LEVELS[random_index(n, LEVELS.len())];
    let message = MESSAGES[random_index(n, MESSAGES.len())];
    let message = format!("{} #{}", message, n % 1000);

    LogEntry {
        level,
        fields: Some(Fields { message, tip: None }),
    }
}

pub struct JournalReader {
    #[cfg(feature = "display_hat")]
    service: String,
    #[cfg(feature = "display_hat")]
    last_cursor: Option<String>,
}

impl JournalReader {
    #[cfg(not(feature = "display_hat"))]
    pub fn new(_service: impl Into<String>) -> Self {
        Self {}
    }

    #[cfg(feature = "display_hat")]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            last_cursor: None,
        }
    }

    #[cfg(not(feature = "display_hat"))]
    pub fn next_lines(&mut self) -> anyhow::Result<Vec<String>> {
        Ok(vec![serde_json::to_string(&random_log_entry()).unwrap()])
    }

    #[cfg(feature = "display_hat")]
    pub fn next_lines(&mut self) -> anyhow::Result<Vec<String>> {
        let mut cmd = Command::new("journalctl");
        cmd.arg("-u")
            .arg(&self.service)
            .arg("--output=short-iso")
            .arg("--show-cursor")
            .arg("--no-pager");

        if let Some(ref cursor) = self.last_cursor {
            cmd.arg("--after-cursor").arg(cursor);
        } else {
            cmd.arg("--since").arg("1 minute ago");
        }

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn journalctl");

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let mut logs = Vec::new();
        let mut last_cursor = None;

        for line_result in reader.lines() {
            let line = line_result?;
            if line.starts_with("-- cursor:") {
                last_cursor = Some(line.trim_start_matches("-- cursor:").trim().to_string());
            } else {
                logs.push(line);
            }
        }

        if let Some(cursor) = last_cursor {
            self.last_cursor = Some(cursor);
        }

        let _ = child.wait()?;
        Ok(logs)
    }
}

pub fn extract_json(line: &str) -> Option<LogEntry> {
    let json_part = line.find('{').map(|i| &line[i..])?;
    serde_json::from_str(json_part).unwrap()
}

pub fn extract_tip_changed(line: &str) -> Option<u64> {
    let entry = extract_json(line)?;
    if let Some(fields) = entry.fields
        && fields.message == "tip_changed"
        && let Some(tip) = fields.tip
        && let Some(slot_str) = tip.split('.').next()
    {
        return slot_str.parse::<u64>().ok();
    }
    None
}
