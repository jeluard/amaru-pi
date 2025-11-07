use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::update::UpdateManager;
use crate::util::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Modal {
    #[default]
    None,
    UpdatePopup(Vec<String>),
}

impl Modal {
    /// Returns `true` if the input was handled, `false` otherwise.
    pub fn handle_input(&mut self, event: InputEvent, update_manager: &mut UpdateManager) -> bool {
        match self {
            Modal::None => false, // Not handled
            Modal::UpdatePopup(_) => {
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
            Modal::UpdatePopup(app_names) => {
                render_update_popup(frame, app_names);
            }
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, Modal::None)
    }
}

fn render_update_popup(frame: &mut Frame, app_names: &[String]) {
    let mut text: Vec<Line> = Vec::new();

    if app_names.is_empty() {
        text.push(Line::from("A system update is available").alignment(Alignment::Center));
    } else if app_names.len() == 1 {
        text.push(Line::from("An update is available for:").alignment(Alignment::Center));
        text.push(
            Line::from(Span::styled(
                app_names[0].clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        );
    } else {
        text.push(Line::from("Updates are available for:").alignment(Alignment::Center));
        let app_list = app_names.join(", ");
        text.push(
            Line::from(Span::styled(
                app_list,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        );
    }

    text.push(Line::from(""));
    text.push(Line::from("Do you want to restart and apply it?").alignment(Alignment::Center));
    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        "[A] Yes, restart now",
        Style::default().fg(Color::Green),
    )]));
    text.push(Line::from(vec![Span::styled(
        "[B] No, remind me in 48 hours",
        Style::default().fg(Color::Yellow),
    )]));

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
