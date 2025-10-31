use qrcode::QrCode;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::{env, time::Duration};
use tui_qrcode::{Colors, QrCodeWidget};

#[derive(Debug, Default)]
pub struct ScanScreen {}

impl crate::screens::Screen for ScanScreen {
    fn display(&mut self, _duration: Duration, frame: &mut Frame) {
        let [_, top_area, _, bottom_area] = Layout::vertical([
            Constraint::Percentage(5),
            Constraint::Percentage(80),
            Constraint::Percentage(5),
            Constraint::Percentage(10),
        ])
        .flex(Flex::Center)
        .areas(frame.area());

        let [top_area] = Layout::horizontal([Constraint::Percentage(80)])
            .flex(Flex::Center)
            .areas(top_area);

        let base_url = "https://jeluard.github.io/amaru-pi/#page=app";
        let url = match env::var("AMARU_WORDS") {
            Ok(words) if !words.is_empty() => format!("{}?words={}", base_url, words),
            _ => base_url.to_string(),
        };

        let qr_code = QrCode::new(url).expect("failed to create QR code");
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
