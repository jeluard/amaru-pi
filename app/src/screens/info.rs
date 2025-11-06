use crate::screens::{AppContext, Kind, Screen, ScreenAction};
use crate::update::{UpdateState, read_state_file};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Displays version information for all managed applications.
pub struct InfoScreen {
    state: UpdateState,
}

impl Default for InfoScreen {
    fn default() -> Self {
        Self {
            // Load the state once on creation
            state: read_state_file().unwrap_or_default(),
        }
    }
}

impl Screen for InfoScreen {
    fn kind(&self) -> Kind {
        Kind::Info
    }

    /// Re-check the state file periodically.
    fn update(&mut self, ac: AppContext) -> ScreenAction {
        // Refresh state every 200 frames (approx 5-10s)
        if ac.frame.frame_count.is_multiple_of(200)
            && let Ok(new_state) = read_state_file()
        {
            self.state = new_state;
        }
        ScreenAction::None
    }

    /// Renders the version information.
    fn display(&self, _ac: AppContext, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();

        if self.state.applications.is_empty() {
            lines.push(Line::from("").centered());
            lines.push(Line::from(" No updates found. ").centered());
            lines.push(Line::from("").centered());
        } else {
            lines.push(Line::from("").centered());
            lines.push(Line::from(" APPLICATION VERSIONS ").centered());
            lines.push(Line::from("").centered());

            for (app_name, app_state) in &self.state.applications {
                lines.push(Line::from(vec![
                    Span::raw("  App:      "),
                    Span::styled(app_name, Style::default().fg(Color::Cyan)),
                ]));
                lines.push(Line::from(vec![
                    Span::raw("  Version:  "),
                    Span::styled(
                        &app_state.current_version,
                        Style::default().fg(Color::Green),
                    ),
                ]));
                if !app_state.pending_version.is_empty() {
                    lines.push(Line::from(vec![
                        Span::raw("  Pending:  "),
                        Span::styled(
                            &app_state.pending_version,
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }
                lines.push(Line::from("")); // spacer
            }
        }

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}
