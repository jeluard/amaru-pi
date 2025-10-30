use amaru_kernel::Slot;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use std::time::{Duration, Instant};
use tui_big_text::{BigText, PixelSize};

use crate::logs::{JournalReader, extract_tip_changed};

pub struct TipScreen {
    reader: JournalReader,
    current_slot: Option<Slot>,
    last_refresh: Instant,
}

impl TipScreen {
    fn update_slot(&mut self, slot: Slot) {
        self.current_slot = Some(slot);
    }
}

impl Default for TipScreen {
    fn default() -> Self {
        let reader = JournalReader::new("amaru.service");
        TipScreen {
            reader,
            current_slot: None,
            last_refresh: Instant::now(),
        }
    }
}

impl crate::screens::Screen for TipScreen {
    fn display(&mut self, _duration: Duration, frame: &mut Frame) {
        let now = Instant::now();
        if now - self.last_refresh > Duration::from_secs(1) {
            self.last_refresh = now;
            let lines = self.reader.next_lines().unwrap_or_default();
            let tips: Vec<_> = lines
                .iter()
                .flat_map(|line| extract_tip_changed(line))
                .collect();
            if let Some(tip) = tips.last() {
                // Set to last tip collected
                self.update_slot((*tip).into());
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(frame.area());

        let lines = self
            .current_slot
            .map(|slot| vec![Line::from("Slot"), format!("#{}", slot).cyan().into()])
            .unwrap_or(vec![Line::from("Bootstrapping")]);
        let text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .centered()
            .lines(lines)
            .build();

        frame.render_widget(text, chunks[1]);
    }
}
