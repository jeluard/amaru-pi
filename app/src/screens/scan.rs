//! A Ratatui example that shows the full range of RGB colors that can be displayed in the terminal.
//!
//! Requires a terminal that supports 24-bit color (true color) and unicode.
//!
//! This example also demonstrates how implementing the Widget trait on a mutable reference
//! allows the widget to update its state while it is being rendered. This allows the fps
//! widget to update the fps calculation and the colors widget to update a cached version of
//! the colors to render instead of recalculating them every frame.
//!
//! This is an alternative to using the `StatefulWidget` trait and a separate state struct. It
//! is useful when the state is only used by the widget and doesn't need to be shared with
//! other widgets.
//!
//! This example runs with the Ratatui library code in the branch that you are currently reading.
//! See the [`latest`] branch for the code which works with the most recent Ratatui release.
//!
//! [`latest`]: https://github.com/ratatui/ratatui/tree/latest

use qrcode::QrCode;
use ratatui::Frame;
use std::time::Duration;
use tui_qrcode::{Colors, QrCodeWidget};

#[derive(Debug, Default)]
pub struct ScanScreen {}

impl crate::screens::Screen for ScanScreen {
    fn display(&mut self, _duration: Duration, frame: &mut Frame) {
        let qr_code = QrCode::new("https://ratatui.rs").expect("failed to create QR code");
        let widget = QrCodeWidget::new(qr_code).colors(Colors::Inverted);
        frame.render_widget(widget, frame.area());
    }
}
