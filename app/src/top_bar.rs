use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct TopBar<'a> {
    pub title: &'a str,
    pub amaru_status_color: Color,
    pub network_status_color: Color,
    pub background: Color,
}

impl<'a> Widget for TopBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [_pad_left, left, before_right, right, _pad_right] = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        Block::default()
            .style(Style::default().bg(self.background))
            .render(area, buf);

        Paragraph::new(Line::from(Span::styled(
            self.title,
            Style::default()
                .fg(Color::White)
                .bg(self.background)
                .add_modifier(Modifier::BOLD),
        )))
        .block(Block::default().borders(Borders::NONE))
        .render(left, buf);

        Paragraph::new(Span::styled("●", Style::default().fg(self.amaru_status_color)))
            .block(Block::default().borders(Borders::NONE))
            .render(before_right, buf);

        Paragraph::new(Span::styled("●", Style::default().fg(self.network_status_color)))
            .block(Block::default().borders(Borders::NONE))
            .render(right, buf);
    }
}
