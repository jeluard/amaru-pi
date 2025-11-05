use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::update::UpdateManager;
use crate::util::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

#[derive(PartialEq, Eq, Default)]
pub enum Modal {
    #[default]
    None,
    UpdatePopup,
}

impl Modal {
    /// Returns `true` if the input was handled, `false` otherwise.
    pub fn handle_input(&mut self, event: InputEvent, update_manager: &mut UpdateManager) -> bool {
        match self {
            Modal::None => false, // Not handled
            Modal::UpdatePopup => {
                match (event.id, event.press_type) {
                    (ButtonId::A, ButtonPress::Short) => {
                        println!("Received update request");
                        UpdateManager::request_update().ok();
                        *self = Modal::None; // Close the modal
                    }
                    (ButtonId::B, ButtonPress::Short) => {
                        println!("Received snooze request");
                        update_manager.snooze().ok();
                        *self = Modal::None; // Close the modal
                    }
                    _ => {}
                }
                true // Handled
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        match self {
            Modal::None => {}
            Modal::UpdatePopup => {
                render_update_popup(frame);
            }
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, Modal::None)
    }
}

fn render_update_popup(frame: &mut Frame) {
    let text = vec![
        Line::from("A system update is available").alignment(Alignment::Center),
        Line::from("Do you want to restart and apply it?").alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[A] Yes, restart now",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            "[B] No, remind me in 48 hours",
            Style::default().fg(Color::Yellow),
        )]),
    ];

    let block = Block::default()
        .title(" System Update ")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);

    let area = centered_rect(80, 40, frame.area());

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}
