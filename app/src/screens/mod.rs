use ratatui::Frame;
use std::time::Duration;

pub mod color;
pub mod exit;
pub mod logo;
pub mod metrics;
pub mod scan;
pub mod tip;
pub mod wifi_settings;

pub trait Screen {
    fn display(&mut self, duration: Duration, frame: &mut Frame);
}
