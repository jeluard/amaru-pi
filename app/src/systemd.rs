use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub enum ActiveState {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Unknown,
}

impl From<&str> for ActiveState {
    fn from(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "inactive" => Self::Inactive,
            "failed" => Self::Failed,
            "activating" => Self::Activating,
            "deactivating" => Self::Deactivating,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum EnabledState {
    Enabled,
    Disabled,
    Static,
    Indirect,
    Generated,
    Masked,
    Unknown,
}

impl From<&str> for EnabledState {
    fn from(s: &str) -> Self {
        match s {
            "enabled" => Self::Enabled,
            "disabled" => Self::Disabled,
            "static" => Self::Static,
            "indirect" => Self::Indirect,
            "generated" => Self::Generated,
            "masked" => Self::Masked,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub active_state: ActiveState,
    pub sub_state: String,
    pub enabled_state: EnabledState,
    pub main_pid: Option<u32>,
}

#[derive(Debug)]
pub enum ServiceError {
    CommandFailed(String),
    ParseError(String),
}

pub fn get_systemd_service_info(service_name: &str) -> Result<ServiceInfo, ServiceError> {
    let output = Command::new("systemctl")
        .arg("show")
        .arg(service_name)
        .arg("--no-pager")
        .arg("--property")
        .arg("Id,Description,ActiveState,SubState,UnitFileState,MainPID")
        .output()
        .map_err(|e| ServiceError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(ServiceError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let map: HashMap<_, _> = stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '=');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();

    let active_state = map
        .get("ActiveState")
        .map(|s| ActiveState::from(s.as_str()))
        .unwrap_or(ActiveState::Unknown);

    let enabled_state = map
        .get("UnitFileState")
        .map(|s| EnabledState::from(s.as_str()))
        .unwrap_or(EnabledState::Unknown);

    let main_pid = map
        .get("MainPID")
        .and_then(|pid_str| pid_str.parse::<u32>().ok())
        .filter(|pid| *pid > 0);

    Ok(ServiceInfo {
        name: map
            .get("Id")
            .cloned()
            .unwrap_or_else(|| service_name.to_string()),
        description: map
            .get("Description")
            .cloned()
            .unwrap_or_else(|| "Unknown".into()),
        active_state,
        sub_state: map
            .get("SubState")
            .cloned()
            .unwrap_or_else(|| "unknown".into()),
        enabled_state,
        main_pid,
    })
}
