use crate::screens::{AppContext, Kind, Screen, ScreenAction};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Displays basic runtime information for the device.
pub struct InfoScreen;

impl Default for InfoScreen {
    fn default() -> Self {
        Self
    }
}

impl Screen for InfoScreen {
    fn kind(&self) -> Kind {
        Kind::Info
    }

    fn update(&mut self, _ac: AppContext) -> ScreenAction {
        ScreenAction::None
    }

    fn display(&self, ac: AppContext, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from("").centered(),
            Line::from(" AMARU PI ").centered(),
            Line::from("").centered(),
            Line::from(vec![
                Span::raw("  Version:  "),
                Span::styled(APP_VERSION, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("  Wi-Fi:    "),
                Span::styled(ac.system.wifi_mode_status.label(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("  Network:  "),
                Span::styled(
                    format!("{:?}", ac.system.network_status.connectivity),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Resolve:  "),
                Span::styled(
                    if ac.system.network_status.resolving {
                        "reachable"
                    } else {
                        "not reachable"
                    },
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}
