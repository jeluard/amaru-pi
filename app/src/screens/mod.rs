use crate::{button::InputEvent, frame::FrameState, systemd::ServiceInfo, wifi::NetworkStatus};
use ratatui::{Frame, layout::Rect};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub mod color;
pub mod exit;
pub mod info;
pub mod logo;
pub mod logs;
pub mod metrics;
pub mod scan;
pub mod tip;
pub mod wifi_settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Color,
    Exit,
    Logo,
    Logs,
    Metrics,
    Scan,
    Tip,
    WiFiSettings,
    Info,
}

impl FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "logo" => Ok(Kind::Logo),
            "tip" => Ok(Kind::Tip),
            "metrics" => Ok(Kind::Metrics),
            "logs" => Ok(Kind::Logs),
            "scan" => Ok(Kind::Scan),
            "info" => Ok(Kind::Info),
            "wifi-settings" | "wifi" | "wifi_settings" => Ok(Kind::WiFiSettings),
            _ => Err(()),
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Color => write!(f, "Color"),
            Kind::Exit => write!(f, "Exit"),
            Kind::Logo => write!(f, "Logo"),
            Kind::Logs => write!(f, "Logs"),
            Kind::Metrics => write!(f, "Metrics"),
            Kind::Scan => write!(f, "Scan"),
            Kind::Tip => write!(f, "Tip"),
            Kind::WiFiSettings => write!(f, "WiFiSettings"),
            Kind::Info => write!(f, "Info"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum WifiConnectionStatus {
    #[default]
    Idle,
    Connecting,
    Success,
    Failed(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum WifiModeStatus {
    #[default]
    StartupProbe,
    HotspotStarting,
    HotspotActive,
    ClientConnecting,
    ClientOnline,
    Recovering,
    Fault(String),
}

impl WifiModeStatus {
    pub fn label(&self) -> &str {
        match self {
            WifiModeStatus::StartupProbe => "Checking uplink",
            WifiModeStatus::HotspotStarting => "Waiting for fallback",
            WifiModeStatus::HotspotActive => "Dedicated hotspot",
            WifiModeStatus::ClientConnecting => "Connecting upstream",
            WifiModeStatus::ClientOnline => "Upstream Wi-Fi",
            WifiModeStatus::Recovering => "Recovering uplink",
            WifiModeStatus::Fault(_) => "Wi-Fi mode fault",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenAction {
    None,
    NextScreen,
    ConnectToWifi(String, String),
    ResetWifiConnectionStatus,
}

#[derive(Debug, Default, Clone)]
pub struct SystemState {
    pub amaru_status: ServiceInfo,
    pub network_status: NetworkStatus,
    pub wifi_connection_status: WifiConnectionStatus,
    pub wifi_mode_status: WifiModeStatus,
}

#[derive(Clone, Copy)]
pub struct AppContext<'a> {
    pub frame: &'a FrameState,
    pub system: &'a SystemState,
}

/// The abstraction allowing to manipulate Screen content
pub trait Screen {
    /// The `Kind` associated to this screen. It must be unique per screen.
    fn kind(&self) -> Kind;

    /// Called just before the first time the Screen is shown
    fn enter(&mut self) {}

    /// Give the opportunity to let this screen handle the `InputEvent`.
    /// If `true` is returned, the event won't be processed further.
    fn handle_input(&mut self, _event: InputEvent) -> bool {
        false
    }

    /// Update the screen's state. Called once per frame *before* display.
    /// Can return a `ScreenAction` to be processed by the `ScreenFlow`.
    fn update(&mut self, _ctx: AppContext) -> ScreenAction {
        ScreenAction::None
    }

    /// Displays this screen. Takes an immutable reference to `self`.
    fn display(&self, ctx: AppContext, f: &mut Frame, area: Rect);

    // Called right after the last time the Screen is shown
    fn exit(&mut self) {}
}
