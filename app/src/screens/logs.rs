use crate::logs::{JournalReader, LogEntry, LogLevel, extract_json};
use crate::screens::{AppContext, Kind, ScreenAction};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use std::cell::RefCell;
use std::time::{Duration, Instant};
use tachyonfx::{CellFilter, EffectManager, EffectTimer, Interpolation, Motion, fx};

impl LogLevel {
    fn color(&self) -> Color {
        match self {
            LogLevel::INFO => Color::Green,
            LogLevel::WARN => Color::Yellow,
            LogLevel::ERROR => Color::Red,
            LogLevel::DEBUG => Color::Blue,
            LogLevel::TRACE => Color::Blue,
        }
    }
}

pub struct LogsScreen {
    reader: JournalReader,
    last_refresh: Instant,
    logs: Vec<LogEntry>,
    effects: RefCell<EffectManager<()>>,
}

impl Default for LogsScreen {
    fn default() -> Self {
        let reader = JournalReader::new("amaru.service");
        LogsScreen {
            reader,
            last_refresh: Instant::now(),
            effects: RefCell::new(EffectManager::default()),
            logs: vec![],
        }
    }
}

impl LogsScreen {
    fn update_logs(&mut self, new_logs: Vec<LogEntry>) {
        // keep most recent logs (up to max)
        let max_items = 25;
        self.logs = new_logs
            .into_iter()
            .take(max_items) // limit new logs to 4
            .chain(self.logs.drain(..)) // append existing logs after
            .take(max_items) // keep only 4 total (newest first)
            .collect();

        self.last_refresh = Instant::now();

        // add smooth slide animation
        self.effects = RefCell::new(EffectManager::default());
        let timer = EffectTimer::from_ms(3000, Interpolation::QuadOut);
        let mut fx_slide = fx::slide_in(Motion::UpToDown, 2, 0, Color::Reset, timer);
        // Optionally filter which cells get this effect
        fx_slide = fx_slide.with_filter(CellFilter::All);
        self.effects.borrow_mut().add_effect(fx_slide);
    }

    fn process_effects(&self, delta: Duration, buf: &mut Buffer, area: Rect) {
        let fx_duration = delta.into();
        self.effects
            .borrow_mut()
            .process_effects(fx_duration, buf, area);
    }
}

fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if s.chars().count() > max_width {
        // handle edge cases for very small widths
        if max_width <= 3 {
            ".".repeat(max_width)
        } else {
            let mut truncated = s.chars().take(max_width - 3).collect::<String>();
            truncated.push_str("...");
            truncated
        }
    } else {
        s.to_string()
    }
}

impl crate::screens::Screen for LogsScreen {
    fn kind(&self) -> Kind {
        Kind::Logs
    }

    fn update(&mut self, ac: AppContext) -> ScreenAction {
        if ac.frame.frame_count.is_multiple_of(100) {
            let logs = self
                .reader
                .next_lines()
                .unwrap_or_default()
                .iter()
                .flat_map(|str| extract_json(str.as_str()))
                .filter(|log| log.level >= LogLevel::INFO)
                .collect::<Vec<_>>();

            if !logs.is_empty() {
                self.update_logs(logs);
            }
        }
        ScreenAction::None
    }

    fn display(&self, _ac: AppContext, frame: &mut Frame, area: Rect) {
        if self.logs.is_empty() {
            // Show "no logs" centered
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(45),
                    Constraint::Length(10),
                    Constraint::Percentage(45),
                ])
                .split(area);

            let para = Paragraph::new("No logs")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(para, chunks[1]);
        } else {
            let max_width = area.width as usize;
            let list_items: Vec<ListItem> = self
                .logs
                .iter()
                .map(|log| {
                    let msg = log
                        .fields
                        .as_ref()
                        .map(|f| f.message.clone())
                        .unwrap_or_default();
                    let line = Line::from(vec![
                        Span::raw("["),
                        Span::styled(
                            format!("{}", log.level),
                            Style::default().fg(log.level.color()),
                        ),
                        Span::raw("] "),
                        Span::raw(truncate_with_ellipsis(
                            &msg,
                            max_width - 3 - log.level.to_string().len(),
                        )),
                    ]);
                    ListItem::new(line)
                })
                .collect();

            let list = List::new(list_items).block(Block::default());

            frame.render_widget(list, area);
        }

        let now = Instant::now();
        let delta = now.duration_since(self.last_refresh);
        self.process_effects(delta, frame.buffer_mut(), area);
    }
}
