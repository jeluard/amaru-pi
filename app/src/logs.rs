use serde::Deserialize;
#[cfg(feature = "display_hat")]
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

#[derive(Debug, Deserialize)]
struct LogEntry {
    fields: Fields,
}

#[derive(Debug, Deserialize)]
struct Fields {
    message: String,
    tip: Option<String>,
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
        Ok(vec![])
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

pub fn extract_tip_changed(line: &str) -> Option<u64> {
    let json_part = line.find('{').map(|i| &line[i..])?;
    let entry: LogEntry = serde_json::from_str(json_part).ok()?;
    if entry.fields.message == "tip_changed"
        && let Some(tip) = entry.fields.tip
        && let Some(slot_str) = tip.split('.').next()
    {
        return slot_str.parse::<u64>().ok();
    }
    None
}
