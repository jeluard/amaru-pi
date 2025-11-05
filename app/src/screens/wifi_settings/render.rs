use super::{ActiveField, Focus, WiFiSettingsScreen};
use crate::{
    screens::{AppContext, WifiConnectionStatus},
    util::centered_rect,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

impl WiFiSettingsScreen {
    pub fn render_layout(&self, ac: AppContext, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Length(3), // Help text
                Constraint::Length(3), // SSID field
                Constraint::Length(3), // Password field + Button
                Constraint::Length(3), // Connect Button
                Constraint::Min(0),    // Keyboard
            ])
            .split(area);

        self.render_instructions(frame, chunks[0]);
        self.render_help_text(frame, chunks[1]);
        self.render_ssid_input(frame, chunks[2]);

        let password_area_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(85), // Password Input
                Constraint::Percentage(15), // Visibility Button
            ])
            .split(chunks[3]);

        self.render_password_input(frame, password_area_chunks[0]);
        self.render_visibility_button(frame, password_area_chunks[1]);
        self.render_connect_button(frame, chunks[4]);
        self.render_keyboard(frame, chunks[5]);

        // Render the popup on top if it's active
        if self.focus == Focus::ConnectingPopup {
            self.render_connecting_popup(ac, frame, area);
        }
    }

    fn render_instructions(&self, frame: &mut Frame, area: Rect) {
        let instruction = Paragraph::new("Enter Wi-Fi credentials for the Pi to connect.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(instruction, area);
    }

    fn render_help_text(&self, frame: &mut Frame, area: Rect) {
        let widget = match self.focus {
            Focus::Fields => {
                let lines = vec![
                    Line::from("X/Y: Change Field").alignment(Alignment::Center),
                    Line::from("A (double): Activate/Toggle").alignment(Alignment::Center),
                ];
                Paragraph::new(lines).alignment(Alignment::Center)
            }
            Focus::Keyboard => {
                let lines = vec![
                    Line::from("A/B/X/Y: Move Cursor").alignment(Alignment::Center),
                    Line::from("A (double): Type | B (double): Backspace")
                        .alignment(Alignment::Center),
                ];
                Paragraph::new(lines)
            }
            Focus::ConnectingPopup => {
                let lines = vec![
                    Line::from("").alignment(Alignment::Center),
                    Line::from("Press any button to dismiss.").alignment(Alignment::Center),
                ];
                Paragraph::new(lines).alignment(Alignment::Center)
            }
        };
        frame.render_widget(widget, area);
    }

    fn render_ssid_input(&self, frame: &mut Frame, area: Rect) {
        self.render_text_input(
            frame,
            area,
            "SSID",
            &self.ssid,
            self.active_field == ActiveField::Ssid,
        );
    }

    fn render_password_input(&self, frame: &mut Frame, area: Rect) {
        let password_display = if self.password_visible {
            self.password.clone()
        } else {
            "*".repeat(self.password.len())
        };
        self.render_text_input(
            frame,
            area,
            "Password",
            &password_display,
            self.active_field == ActiveField::Password,
        );
    }

    fn render_keyboard(&self, frame: &mut Frame, area: Rect) {
        if self.focus == Focus::Keyboard {
            self.keyboard.render(frame, area);
        }
    }

    fn render_visibility_button(&self, frame: &mut Frame, area: Rect) {
        let text = if self.password_visible {
            "Hide"
        } else {
            "Show"
        };

        let is_active =
            self.active_field == ActiveField::PasswordVisibility && self.focus == Focus::Fields;
        let style = if is_active {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default().borders(Borders::ALL).style(style);
        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn render_connect_button(&self, frame: &mut Frame, area: Rect) {
        let text = "[ Connect ]";
        let is_active =
            self.active_field == ActiveField::ConnectButton && self.focus == Focus::Fields;
        let style = if is_active {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };

        let block = Block::default().borders(Borders::ALL).style(style);
        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn render_text_input(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        text: &str,
        is_active: bool,
    ) {
        let style = if is_active && (self.focus == Focus::Fields || self.focus == Focus::Keyboard) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(style);
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_connecting_popup(&self, ac: AppContext, frame: &mut Frame, area: Rect) {
        let (text, style) = match &ac.system.wifi_connection_status {
            WifiConnectionStatus::Idle | WifiConnectionStatus::Connecting => (
                "Connecting...".to_string(),
                Style::default().fg(Color::Yellow),
            ),
            WifiConnectionStatus::Success => (
                "Success! Connected to Wi-Fi.".to_string(),
                Style::default().fg(Color::Green),
            ),
            WifiConnectionStatus::Failed(e) => (
                format!("Connection Failed:\n{}", e),
                Style::default().fg(Color::Red),
            ),
        };

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(text, style)).alignment(Alignment::Center),
            Line::from(""),
            Line::from("Press any button to dismiss.").alignment(Alignment::Center),
        ];

        let block = Block::default()
            .title(" Wi-Fi Connection ")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        let popup_area = centered_rect(80, 40, area);

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }
}
