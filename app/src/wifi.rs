use anyhow::{Context, anyhow};
use std::{
    ffi::{OsStr, OsString},
    net::{TcpStream, ToSocketAddrs},
    process::{Command, Stdio},
    time::Duration,
};

#[cfg(feature = "display_hat")]
const WIFI_INTERFACE: &str = "wlan0";

#[cfg(feature = "display_hat")]
const CONNECTION_NAME: &str = "mobile";

#[cfg(feature = "display_hat")]
const DEFAULT_HOTSPOT_CONNECTION_NAME: &str = "amaru-hotspot";

#[cfg(feature = "display_hat")]
const DEFAULT_HOTSPOT_SSID: &str = "Amaru Setup";

#[cfg(feature = "display_hat")]
const DEFAULT_HOTSPOT_PASSWORD: &str = "amaru-setup";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WifiOperatingMode {
    #[default]
    Unknown,
    Disconnected,
    Client,
    Hotspot,
}

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

pub fn is_port_open<A: ToSocketAddrs>(addr: A) -> anyhow::Result<bool> {
    let timeout = Duration::from_secs(2);
    let Some(target) = addr.to_socket_addrs()?.next() else {
        return Ok(false);
    };

    Ok(TcpStream::connect_timeout(&target, timeout).is_ok())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    #[default]
    Unknown,
    Connected,
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
            "connected" => NetworkState::Connected,
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
    pub resolving: bool,
}

pub fn run_and_capture<I, S>(program: &str, args: I) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<OsString> = args
        .into_iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect();

    let mut cmd = Command::new(program);
    cmd.args(&args);
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
fn run_with_timeout<I, S>(
    program: &str,
    args: I,
    timeout: Duration,
    action: &str,
) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<OsString> = args
        .into_iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect();

    let mut child = Command::new(program)
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to start {action}"))?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                if status.success() {
                    return Ok(());
                }

                return Err(anyhow!("{action} failed: {status}"));
            }
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(anyhow!(
                        "{action} timed out after {}s",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

#[cfg(feature = "display_hat")]
fn hotspot_connection_name() -> String {
    std::env::var("AMARU_HOTSPOT_CONNECTION")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_HOTSPOT_CONNECTION_NAME.to_string())
}

#[cfg(feature = "display_hat")]
fn hotspot_ssid() -> String {
    std::env::var("AMARU_HOTSPOT_SSID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_HOTSPOT_SSID.to_string())
}

#[cfg(feature = "display_hat")]
fn hotspot_password() -> String {
    std::env::var("AMARU_HOTSPOT_PASSWORD")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_HOTSPOT_PASSWORD.to_string())
}

#[cfg(feature = "display_hat")]
fn radio_on() -> anyhow::Result<()> {
    run_and_capture("nmcli", ["radio", "wifi", "on"])?;
    Ok(())
}

#[cfg(feature = "display_hat")]
fn connection_exists(name: &str) -> bool {
    run_and_capture(
        "nmcli",
        vec![
            "-g".to_string(),
            "NAME".to_string(),
            "connection".to_string(),
            "show".to_string(),
            name.to_string(),
        ],
    )
    .is_ok()
}

#[cfg(feature = "display_hat")]
fn disconnect_device(timeout: Duration) -> anyhow::Result<()> {
    run_with_timeout(
        "nmcli",
        ["device", "disconnect", WIFI_INTERFACE],
        timeout,
        "disconnect wifi device",
    )
}

#[cfg(feature = "display_hat")]
fn current_connection_name() -> anyhow::Result<Option<String>> {
    let stdout = run_and_capture(
        "nmcli",
        ["-g", "GENERAL.CONNECTION", "device", "show", WIFI_INTERFACE],
    )?;

    let name = stdout.lines().next().unwrap_or_default().trim();
    if name.is_empty() || name == "--" {
        Ok(None)
    } else {
        Ok(Some(name.to_string()))
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

    let resolving = std::env::var("AMARU_PEER_ADDRESS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(is_port_open)
        .transpose()?
        .unwrap_or(false);

    Ok(NetworkStatus {
        state: parts[0].into(),
        connectivity: parts[1].into(),
        resolving,
    })
}

#[cfg(not(feature = "display_hat"))]
pub fn check_network_status() -> Result<NetworkStatus, Box<dyn std::error::Error>> {
    Ok(NetworkStatus {
        state: NetworkState::ConnectedGlobal,
        connectivity: Connectivity::Full,
        resolving: true,
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
pub fn delete_connection() -> anyhow::Result<()> {
    // Ignore failure
    let _ = run_and_capture("nmcli", ["con", "delete", CONNECTION_NAME].to_vec());

    Ok(())
}

#[cfg(not(feature = "display_hat"))]
pub fn delete_connection() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn set_connection(ssid: &str, password: &str) -> anyhow::Result<()> {
    radio_on()?;
    delete_connection()?;

    run_and_capture(
        "nmcli",
        [
            "dev",
            "wifi",
            "connect",
            ssid,
            "password",
            password,
            "ifname",
            WIFI_INTERFACE,
            "name",
            CONNECTION_NAME,
        ]
        .to_vec(),
    )?;

    Ok(())
}

#[cfg(not(feature = "display_hat"))]
pub fn set_connection(_ssid: &str, _password: &str) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn up_connection(timeout: Duration) -> anyhow::Result<()> {
    run_with_timeout(
        "nmcli",
        ["con", "up", CONNECTION_NAME],
        timeout,
        "bring up wifi connection",
    )
}

#[cfg(not(feature = "display_hat"))]
pub fn up_connection(_timeout: Duration) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn down_connection(timeout: Duration) -> anyhow::Result<()> {
    run_with_timeout(
        "nmcli",
        ["con", "down", CONNECTION_NAME],
        timeout,
        "bring down wifi connection",
    )
}

#[cfg(not(feature = "display_hat"))]
pub fn down_connection(_timeout: Duration) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn current_operating_mode() -> anyhow::Result<WifiOperatingMode> {
    let Some(connection_name) = current_connection_name()? else {
        return Ok(WifiOperatingMode::Disconnected);
    };

    if connection_name == hotspot_connection_name() {
        Ok(WifiOperatingMode::Hotspot)
    } else {
        Ok(WifiOperatingMode::Client)
    }
}

#[cfg(not(feature = "display_hat"))]
pub fn current_operating_mode() -> anyhow::Result<WifiOperatingMode> {
    Ok(WifiOperatingMode::Client)
}

#[cfg(feature = "display_hat")]
pub fn ensure_hotspot_profile() -> anyhow::Result<()> {
    radio_on()?;

    let connection_name = hotspot_connection_name();
    let ssid = hotspot_ssid();
    let password = hotspot_password();

    if password.len() < 8 {
        return Err(anyhow!(
            "AMARU_HOTSPOT_PASSWORD must be at least 8 characters long"
        ));
    }

    if !connection_exists(&connection_name) {
        run_and_capture(
            "nmcli",
            vec![
                "connection".to_string(),
                "add".to_string(),
                "type".to_string(),
                "wifi".to_string(),
                "ifname".to_string(),
                WIFI_INTERFACE.to_string(),
                "con-name".to_string(),
                connection_name.clone(),
                "autoconnect".to_string(),
                "no".to_string(),
                "ssid".to_string(),
                ssid.clone(),
            ],
        )?;
    }

    run_and_capture(
        "nmcli",
        vec![
            "connection".to_string(),
            "modify".to_string(),
            connection_name,
            "connection.autoconnect".to_string(),
            "no".to_string(),
            "connection.interface-name".to_string(),
            WIFI_INTERFACE.to_string(),
            "802-11-wireless.mode".to_string(),
            "ap".to_string(),
            "802-11-wireless.band".to_string(),
            "bg".to_string(),
            "802-11-wireless.ssid".to_string(),
            ssid,
            "ipv4.method".to_string(),
            "shared".to_string(),
            "ipv6.method".to_string(),
            "ignore".to_string(),
            "wifi-sec.key-mgmt".to_string(),
            "wpa-psk".to_string(),
            "wifi-sec.psk".to_string(),
            password,
        ],
    )?;

    Ok(())
}

#[cfg(not(feature = "display_hat"))]
pub fn ensure_hotspot_profile() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn start_hotspot(timeout: Duration) -> anyhow::Result<()> {
    ensure_hotspot_profile()?;
    let _ = disconnect_device(Duration::from_secs(10));

    run_with_timeout(
        "nmcli",
        vec![
            "connection".to_string(),
            "up".to_string(),
            hotspot_connection_name(),
        ],
        timeout,
        "start hotspot",
    )
}

#[cfg(not(feature = "display_hat"))]
pub fn start_hotspot(_timeout: Duration) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "display_hat")]
pub fn stop_hotspot(timeout: Duration) -> anyhow::Result<()> {
    let _ = run_with_timeout(
        "nmcli",
        vec![
            "connection".to_string(),
            "down".to_string(),
            hotspot_connection_name(),
        ],
        timeout,
        "stop hotspot",
    );

    Ok(())
}

#[cfg(not(feature = "display_hat"))]
pub fn stop_hotspot(_timeout: Duration) -> anyhow::Result<()> {
    Ok(())
}
