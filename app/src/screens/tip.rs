use amaru_doctor::model::otel_view::OtelViewState;
use amaru_doctor::model::prom_metrics::PromMetricsViewState;
use amaru_doctor::otel::service::OtelCollectorService;
use amaru_doctor::otel::TraceGraphSnapshot;
use amaru_doctor::prometheus::model::Timestamp;
use amaru_doctor::prometheus::service::{MetricsPoller, MetricsPollerHandle};
/// A Ratatui example that demonstrates how to handle charts.
///
/// This example demonstrates how to draw various types of charts such as line,
/// bar, and scatter charts.
///
/// This example runs with the Ratatui library code in the branch that you are
/// currently reading. See the [`latest`] branch for the code which works with
/// the most recent Ratatui release.
///
/// [`latest`]: https://github.com/ratatui/ratatui/tree/latest
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::{self, Marker};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, LegendPosition, Paragraph, Wrap};
use std::time::Duration;

pub struct TipScreen {
    otel_view: OtelViewState,
}

impl Default for TipScreen {
    fn default() -> Self {
        let otel_service = OtelCollectorService::new("0.0.0.0:4317");
        let otel_handle = otel_service.start();
        let otel_view = OtelViewState::new(otel_handle.snapshot);
        
        TipScreen {
            otel_view,
        }
    }
}

impl crate::screens::Screen for TipScreen {
    fn display(&mut self, duration: Duration, frame: &mut Frame) {
        self.otel_view.sync_state();
        println!("{:?}", self.otel_view.last_synced_data);
        let name = if let Some(value) =  self.otel_view.trace_graph_source.load().spans.values().next() {
            value.name.clone()
        } else {
            "1936839".to_string()
        };
        let text = vec![
            Line::from(Span::styled(format!("Slot #{}", name), Style::default())),
            Line::from(Span::styled("This is centered text.", Style::default())),
        ];

        // Create a paragraph widget
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center) // center horizontally
            .wrap(Wrap { trim: true })
            .block(Block::default());

        let area = frame.area();
        let text_height = 2;
        let y_offset = area.y + (area.height.saturating_sub(text_height)) / 2;
        let rect = Rect {
            x: area.x,
            y: y_offset,
            width: area.width,
            height: text_height,
        };

        frame.render_widget(paragraph, rect);
    }
}
