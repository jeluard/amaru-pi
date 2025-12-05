use crate::logs::{JournalReader, extract_new_tip, extract_tip_changed};
use crate::screens::{AppContext, Kind, ScreenAction};
use crate::wifi::Connectivity;
use amaru_kernel::Slot;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use std::time::{Duration, Instant};
use tui_big_text::{BigText, PixelSize};

pub struct TipScreen {
    reader: JournalReader,
    current_slot: Option<(Slot, bool)>,
    last_refresh: Instant,
}

impl TipScreen {
    fn update_slot(&mut self, slot: (Slot, bool)) {
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

fn create_lines<'a>(ac: AppContext, current_slot: Option<(Slot, bool)>) -> (Vec<Line<'a>>, bool) {
    if ac.system.network_status.connectivity != Connectivity::Full {
        (vec![Line::from("Not connected")], false)
    } else if !ac.system.network_status.resolving {
        (vec![Line::from("Not resolving")], false)
    } else if let Some((current_slot, synced)) = current_slot {
        (
            vec![
                Line::from("Slot"),
                if synced {
                    format!("#{}", current_slot).green().into()
                } else {
                    format!("#{}", current_slot).cyan().into()
                },
            ],
            false,
        )
    } else {
        (vec![Line::from("Bootstrapping")], true)
    }
}

impl crate::screens::Screen for TipScreen {
    fn kind(&self) -> Kind {
        Kind::Tip
    }

    fn update(&mut self, _ac: AppContext) -> ScreenAction {
        let now = Instant::now();
        if now - self.last_refresh > Duration::from_secs(1) {
            self.last_refresh = now;
            let lines = self.reader.next_lines().unwrap_or_default();
            let new_tips: Vec<_> = lines
                .iter()
                .flat_map(|line| extract_new_tip(line))
                .collect();
            if let Some(tip) = new_tips.last() {
                // Set to last tip collected
                self.update_slot(((*tip).into(), true));
            } else {
                let tips: Vec<_> = lines
                    .iter()
                    .flat_map(|line| extract_tip_changed(line))
                    .collect();
                if let Some(tip) = tips.last() {
                    // Set to last tip collected
                    self.update_slot(((*tip).into(), false));
                }
            }
        }
        ScreenAction::None
    }

    fn display(&self, ac: AppContext, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ])
            .split(area);

        let (lines, details) = create_lines(ac, self.current_slot);
        let text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .centered()
            .lines(lines)
            .build();

        frame.render_widget(text, chunks[1]);

        if details {
            let details_line = Line::from("this may take a couple minutes").centered();
            frame.render_widget(details_line, chunks[2]);
        }
    }
}
