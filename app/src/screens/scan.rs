//! A Ratatui example that shows the full range of RGB colors that can be
//! displayed in the terminal.
//!
//! Requires a terminal that supports 24-bit color (true color) and unicode.
//!
//! This example also demonstrates how implementing the Widget trait on a
//! mutable reference allows the widget to update its state while it is being
//! rendered. This allows the fps widget to update the fps calculation and the
//! colors widget to update a cached version of the colors to render instead of
//! recalculating them every frame.
//!
//! This is an alternative to using the `StatefulWidget` trait and a separate
//! state struct. It is useful when the state is only used by the widget and
//! doesn't need to be shared with other widgets.
//!
//! This example runs with the Ratatui library code in the branch that you are
//! currently reading. See the [`latest`] branch for the code which works with
//! the most recent Ratatui release.
//!
//! [`latest`]: https://github.com/ratatui/ratatui/tree/latest

use qrcode::QrCode;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::time::Duration;
use tui_qrcode::{Colors, QrCodeWidget};

#[derive(Debug, Default)]
pub struct ScanScreen {}

impl crate::screens::Screen for ScanScreen {
    fn display(&mut self, _duration: Duration, frame: &mut Frame) {
        let [_, top_area, _, bottom_area] = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(60),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ])
        .flex(Flex::Center)
        .areas(frame.area());

        let [top_area] = Layout::horizontal([Constraint::Percentage(60)])
            .flex(Flex::Center)
            .areas(top_area);

        let qr_code = QrCode::new("https://jeluard.github.io/amaru-pi/?page=app").expect("failed to create QR code");
        let widget = QrCodeWidget::new(qr_code).colors(Colors::Inverted);
        frame.render_widget(widget, top_area);

        // Add centered text below
        let text = Paragraph::new(Line::from(Span::styled(
            "Scan to configure the PI",
            Style::default().fg(Color::Yellow),
        )))
        .alignment(Alignment::Center);

        frame.render_widget(text, bottom_area);
    }
}
