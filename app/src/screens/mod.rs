use ratatui::Frame;
use std::{
    fmt::{self, Display},
    time::Duration,
};

use crate::{button::InputEvent, wifi::Connectivity};

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
    elapsed: Duration,
    connectivity: Connectivity,
}

impl State {
    pub fn new(elapsed: Duration, connectivity: Connectivity) -> Self {
        State {
            elapsed,
            connectivity,
        }
    }
}

/// The abstraction allowing to manipulate Screen content
pub trait Screen {
    fn kind(&self) -> Kind;

    // Called just before the first time the Screen is shown
    fn enter(&mut self) {}

    fn handle_input(&mut self, _event: InputEvent) -> bool {
        false
    }

    fn display(&mut self, state: State, frame: &mut Frame) -> bool;

    // Called right after the last time the Screen is shown
    fn exit(&mut self) {}
}
