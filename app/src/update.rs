use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const STATE_FILE_PATH: &str = "/home/pi/.amaru_update_state.json";
const UPDATE_TRIGGER_PATH: &str = "/home/pi/.update_requested";
const SNOOZE_DURATION_SECS: u64 = 48 * 60 * 60; // 48 hours

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct AppUpdateState {
    #[serde(default)]
    current_version: String,
    #[serde(default)]
    pending_version: String,
    #[serde(default)]
    staged_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateState {
    #[serde(default)]
    notify_after: u64,
    #[serde(default)]
    applications: HashMap<String, AppUpdateState>,
}

impl UpdateState {
    /// Checks if any application has a pending update ready.
    fn is_update_available(&self) -> bool {
        self.applications.values().any(|app_state| {
            // An update is available if a pending version and staged path are set.
            !app_state.pending_version.is_empty() && !app_state.staged_path.is_empty()
        })
    }

    /// Checks if the user has requested to snooze notifications.
    fn is_snoozed(&self) -> bool {
        let now = current_timestamp().unwrap_or(0);
        now < self.notify_after
    }
}

#[derive(PartialEq, Eq)]
pub enum UpdateStatus {
    Idle,
    UpdateReadyToNotify,
}

pub struct UpdateManager {
    last_check: Instant,
    interval: Duration,
    current_state: UpdateState,
}

impl UpdateManager {
    pub fn new(interval: Duration) -> Self {
        Self {
            last_check: Instant::now() - interval, // Force check on first run
            current_state: Self::read_state_file().unwrap_or_default(),
            interval,
        }
    }

    pub fn check_for_update(&mut self) -> UpdateStatus {
        if self.last_check.elapsed() >= self.interval {
            self.last_check = Instant::now();
            match Self::read_state_file() {
                Ok(new_state) => self.current_state = new_state,
                Err(e) => println!("Error reading state file {}: {}", STATE_FILE_PATH, e),
            }
        }

        if self.current_state.is_update_available() && !self.current_state.is_snoozed() {
            UpdateStatus::UpdateReadyToNotify
        } else {
            UpdateStatus::Idle
        }
    }

    /// Snoozes notifications by updating the `notify_after` timestamp in the state file.
    pub fn snooze(&mut self) -> Result<()> {
        let now = current_timestamp()?;
        self.current_state.notify_after = now + SNOOZE_DURATION_SECS;
        Self::write_state_file(&self.current_state)?;
        self.last_check = Instant::now(); // Update cache time
        Ok(())
    }

    /// Triggers the update by creating the trigger file.
    pub fn request_update() -> Result<()> {
        fs::File::create(UPDATE_TRIGGER_PATH)?;
        Ok(())
    }

    fn read_state_file() -> Result<UpdateState> {
        let path = Path::new(STATE_FILE_PATH);
        if !path.exists() {
            println!("Warning, no state file found {}", STATE_FILE_PATH);
            return Ok(UpdateState::default());
        }
        let data = fs::read_to_string(path)?;
        let state: UpdateState = serde_json::from_str(&data)?;
        Ok(state)
    }

    fn write_state_file(state: &UpdateState) -> Result<()> {
        let path = Path::new(STATE_FILE_PATH);
        let data = serde_json::to_string_pretty(state)?;
        fs::write(path, data)?;
        Ok(())
    }
}

fn current_timestamp() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
