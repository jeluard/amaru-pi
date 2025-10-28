use amaru_doctor::model::otel_view::OtelViewState;
use amaru_doctor::model::prom_metrics::PromMetricsViewState;
use amaru_doctor::otel::TraceGraphSnapshot;
use amaru_doctor::otel::service::OtelCollectorService;
use amaru_doctor::prometheus::model::Timestamp;
use amaru_doctor::prometheus::service::{MetricsPoller, MetricsPollerHandle};
use amaru_kernel::Slot;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::{self, Marker};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, LegendPosition, Paragraph, Wrap};
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tui_big_text::{BigText, PixelSize};

use crate::logs::extract_tip_changed;

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

pub struct JournalReader {
    service: String,
    last_cursor: Option<String>,
}

impl JournalReader {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            last_cursor: None,
        }
    }

    pub fn read_logs(&mut self) -> anyhow::Result<Vec<String>> {
        let mut cmd = Command::new("journalctl");
        cmd.arg("-u")
            .arg(&self.service)
            .arg("--output=short-iso")
            .arg("--show-cursor")
            .arg("--no-pager");

        if let Some(ref cursor) = self.last_cursor {
            cmd.arg("--after-cursor").arg(cursor);
        } else {
            cmd.arg("--since").arg("1 minute ago");
        }

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn journalctl");

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let mut logs = Vec::new();
        let mut last_cursor = None;

        for line_result in reader.lines() {
            let line = line_result?;
            if line.starts_with("-- cursor:") {
                last_cursor = Some(line.trim_start_matches("-- cursor:").trim().to_string());
            } else {
                logs.push(line);
            }
        }

        if let Some(cursor) = last_cursor {
            self.last_cursor = Some(cursor);
        }

        let _ = child.wait()?;
        Ok(logs)
    }
}

impl crate::screens::Screen for TipScreen {
    fn display(&mut self, duration: Duration, frame: &mut Frame) {
        let now = Instant::now();
        if now - self.last_refresh > Duration::from_secs(1) {
            self.last_refresh = now;
            let lines = self.reader.read_logs().unwrap_or_default();
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
