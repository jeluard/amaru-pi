use ratatui::{Frame, layout::Rect};
use std::{
    fmt::{self, Display},
    time::Duration,
};

use crate::{button::InputEvent, systemd::ServiceInfo, wifi::NetworkStatus};

pub mod color;
pub mod exit;
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
        }
    }
}

pub struct State {
    pub frame_count: u64,
    pub elapsed_since_startup: Duration,
    pub elapsed_since_last_frame: Duration,
    pub amaru_status: ServiceInfo,
    pub network_status: NetworkStatus,
}

impl State {
    pub fn new(
        frame_count: u64,
        elapsed_since_last_frame: Duration,
        elapsed_since_startup: Duration,
        amaru_status: ServiceInfo,
        network_status: NetworkStatus,
    ) -> Self {
        State {
            frame_count,
            elapsed_since_startup,
            elapsed_since_last_frame,
            amaru_status,
            network_status,
        }
    }
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

    /// Displays this screen.
    /// Will be called again while `true` is returned. If `false`, triggers the
    /// logic to change screen.
    fn display(&mut self, state: State, area: &mut Frame, area: Rect) -> bool;

    // Called right after the last time the Screen is shown
    fn exit(&mut self) {}
}
