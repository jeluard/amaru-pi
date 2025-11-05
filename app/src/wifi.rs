use anyhow::{Context, anyhow};
use std::{
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Connectivity {
    #[default]
    Unknown,
    None,
    Portal,
    Limited,
    Full,
}

impl From<&str> for Connectivity {
    fn from(s: &str) -> Self {
        match s.trim() {
            "none" => Connectivity::None,
            "limited" => Connectivity::Limited,
            "full" => Connectivity::Full,
            "portal" => Connectivity::Portal,
            _ => Connectivity::Unknown,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    #[default]
    Unknown,
    ConnectedGlobal,
    ConnectedLocal,
    ConnectedSite,
    Connecting,
    Disconnected,
    Disconnecting,
}

impl From<&str> for NetworkState {
    fn from(s: &str) -> Self {
        match s.trim() {
            "connected-global" => NetworkState::ConnectedGlobal,
            "connected-local" => NetworkState::ConnectedLocal,
            "connected-site" => NetworkState::ConnectedSite,
            "connecting" => NetworkState::Connecting,
            "disconnected" => NetworkState::Disconnected,
            "disconnecting" => NetworkState::Disconnecting,
            _ => NetworkState::Unknown,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NetworkStatus {
    pub state: NetworkState,
    pub connectivity: Connectivity,
}

pub fn run_and_capture(program: &str, args: Vec<&str>) -> anyhow::Result<String> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = cmd.output().context("failed to spawn command")?;
    let stdout = String::from_utf8_lossy(&child.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&child.stderr).trim().to_string();

    if child.status.success() {
        Ok(stdout)
    } else {
        Err(anyhow!(
            "command exited with status {}: {} :: {}",
            child.status,
            stdout,
            stderr
        ))
    }
}

#[cfg(feature = "display_hat")]
pub fn check_network_status() -> anyhow::Result<NetworkStatus> {
    let stdout = run_and_capture(
        "nmcli",
        ["-t", "-f", "STATE,CONNECTIVITY", "general", "status"].to_vec(),
    )?;
    let parts: Vec<&str> = stdout.split(':').collect();

    if parts.len() < 2 {
        return Err(anyhow!(format!("unexpected nmcli output: {}", stdout),));
    }

    Ok(NetworkStatus {
        state: parts[0].into(),
        connectivity: parts[1].into(),
    })
}

#[cfg(not(feature = "display_hat"))]
pub fn check_network_status() -> Result<NetworkStatus, Box<dyn std::error::Error>> {
    Ok(NetworkStatus {
        state: NetworkState::ConnectedGlobal,
        connectivity: Connectivity::Full,
    })
}

#[derive(Debug)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub mode: String,
    pub channel: u32,
    pub rate: String,
    pub signal: u8, // 0-100
    pub bars: String,
    pub security: String,
}

#[cfg(feature = "display_hat")]
pub fn scan_ssids() -> anyhow::Result<Vec<WifiNetwork>> {
    let stdout = run_and_capture(
        "nmcli",
        [
            "-t",
            "-f",
            "SSID,BSSID,MODE,CHAN,RATE,SIGNAL,BARS,SECURITY",
            "dev",
            "wifi",
            "list",
        ]
        .to_vec(),
    )?;

    let mut networks = Vec::new();

    for line in stdout.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 8 {
            continue; // skip malformed lines
        }

        let channel = fields[3].parse::<u32>().unwrap_or(0);
        let signal = fields[5].parse::<u8>().unwrap_or(0);

        networks.push(WifiNetwork {
            ssid: fields[0].to_string(),
            bssid: fields[1].to_string(),
            mode: fields[2].to_string(),
            channel,
            rate: fields[4].to_string(),
            signal,
            bars: fields[6].to_string(),
            security: fields[7].to_string(),
        });
    }

    Ok(networks)
}

#[cfg(not(feature = "display_hat"))]
pub fn scan_ssids() -> anyhow::Result<Vec<String>> {
    Ok(vec![])
}

#[cfg(feature = "display_hat")]
const CONNECTION_NAME: &str = "mobile";

#[cfg(feature = "display_hat")]
pub fn set_connection(ssid: &str, password: &str) -> anyhow::Result<()> {
    run_and_capture(
        "nmcli",
        [
            "con", "add", "type", "wifi", "ifname", "wlan0", "con-name", "mobile", "ssid", ssid,
        ]
        .to_vec(),
    )?;

    run_and_capture(
        "nmcli",
        [
            "con",
            "modify",
            CONNECTION_NAME,
            "wifi-sec.key-mgmt",
            "wpa-psk",
        ]
        .to_vec(),
    )?;

    run_and_capture(
        "nmcli",
        ["con", "modify", CONNECTION_NAME, "wifi-sec.psk", password].to_vec(),
    )?;
    Ok(())
}

#[cfg(not(feature = "display_hat"))]
pub fn set_connection(_ssid: &str, _password: &str) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn up_connection(timeout: Duration) -> anyhow::Result<()> {
    let mut child = Command::new("nmcli")
        .args(["con", "up", CONNECTION_NAME])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to start nmcli up")?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                if status.success() {
                    return Ok(());
                } else {
                    return Err(anyhow!("nmcli con up failed: {}", status));
                }
            }
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(anyhow!(
                        "timed out after {}s while bringing up connection",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

#[cfg(not(feature = "display_hat"))]
pub fn up_connection(_timeout: Duration) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn down_connection(timeout: Duration) -> anyhow::Result<()> {
    let mut child = Command::new("nmcli")
        .args(["con", "down", CONNECTION_NAME])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to start nmcli down")?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                if status.success() {
                    println!("Connection successfully deactivated.");
                    return Ok(());
                } else {
                    return Err(anyhow!("nmcli con down failed: {}", status));
                }
            }
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(anyhow!(
                        "timed out after {}s while bringing down connection",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

#[cfg(not(feature = "display_hat"))]
pub fn down_connection(_timeout: Duration) -> anyhow::Result<()> {
    Ok(())
}
