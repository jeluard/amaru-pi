use super::{KeyboardMode, KeyboardWidget};
use crate::keyboard::layout::KEYBOARD_LAYOUT;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

impl KeyboardWidget {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let rows = self.build_rows();
        let keyboard_widget =
            Paragraph::new(rows).block(Block::default().borders(Borders::ALL).title("Keyboard"));
        frame.render_widget(keyboard_widget, area);
    }

    fn build_rows(&self) -> Vec<Line<'_>> {
        KEYBOARD_LAYOUT
            .iter()
            .enumerate()
            .map(|(row_idx, row)| self.build_row_line(row_idx, row))
            .collect()
    }

    fn build_row_line(&self, row_idx: usize, row: &[&str]) -> Line<'_> {
        let mut spans: Vec<Span> = row
            .iter()
            .enumerate()
            .map(|(col_idx, key)| self.build_key_span(row_idx, col_idx, key))
            .collect();

        let indent = match row_idx {
            0 => "       ",
            1 => "        ",
            2 => "   ",
            3 => "   ",
            4 => "            ",
            _ => "",
        };
        spans.insert(0, Span::raw(indent));

        Line::from(spans).alignment(Alignment::Left)
    }

    fn build_key_span(&self, row_idx: usize, col_idx: usize, key: &str) -> Span<'_> {
        let is_shifted = matches!(self.mode, KeyboardMode::Shift | KeyboardMode::CapsLock);
        let key_str = self.get_key_display_string(key, is_shifted);
        let style = self.get_key_style(row_idx, col_idx, key);
        Span::styled(format!(" {} ", key_str), style)
    }

    pub(super) fn get_key_display_string(&self, key: &str, is_shifted: bool) -> String {
        if !is_shifted {
            return key.to_string();
        }
        // The key, char or symbol, is shifted

        if let Some(symbol) = self.shifted_symbols.get(key) {
            return symbol.to_string();
        }
        // It's not a symbol, it's a char

        let is_letter = key.len() == 1 && key.chars().next().unwrap().is_alphabetic();
        if is_letter {
            return key.to_uppercase();
        }

        // Fallback
        key.to_string()
    }

    fn get_key_style(&self, row_idx: usize, col_idx: usize, key: &str) -> Style {
        if (row_idx, col_idx) == self.cursor {
            return Style::default().bg(Color::Yellow).fg(Color::Black);
        }
        if key == "caps" && self.mode == KeyboardMode::CapsLock {
            return Style::default().bg(Color::Cyan).fg(Color::Black);
        }
        if key == "shift" && self.mode == KeyboardMode::Shift {
            return Style::default().bg(Color::Cyan).fg(Color::Black);
        }
        Style::default().fg(Color::White)
    }
}
