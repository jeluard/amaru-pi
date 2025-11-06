use crate::{
    metrics_data::MetricData,
    screens::{AppContext, Kind, Screen, ScreenAction},
};
use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};
use std::iter;

pub struct MetricsScreen;

impl Default for MetricsScreen {
    fn default() -> Self {
        Self
    }
}

/// Helper to add padding and generate 5 labels for the Y-axis
fn get_padded_y_bounds(bounds: [f64; 2]) -> [f64; 2] {
    let [min, max] = bounds;
    let (min_padded, max_padded) = if (max - min).abs() < f64::EPSILON {
        // If the line is flat, create a small window around it
        (min - 1.0, max + 1.0)
    } else {
        // Add 10% padding
        let range = max - min;
        let padding = range * 0.1;
        (min - padding, max + padding)
    };

    [min_padded, max_padded]
}

struct ChartDatasetConfig<'a> {
    data: Option<&'a MetricData>,
    label: &'a str,
    color: Color,
}

const ONE_KB: f64 = 1024.0;
const ONE_MB: f64 = 1024.0 * ONE_KB;
const ONE_GB: f64 = 1024.0 * ONE_MB;

fn format_mib_label(value: f64) -> String {
    if value >= ONE_GB {
        format!("{:.0} GiB", value / ONE_GB)
    } else if value >= ONE_MB {
        format!("{:.0} MiB", value / ONE_MB)
    } else {
        format!("{:.0} B", value / ONE_KB)
    }
}

pub enum MetricKind {
    Bytes,
    Percentage,
}

fn adjust_vec(v: Vec<f64>) -> Vec<f64> {
    assert!(v.len() >= 3, "Vector must have at least 3 elements");

    let first = *v.first().unwrap();
    let last = *v.last().unwrap();

    let filtered_middle = v[1..v.len() - 1].iter().fold(Vec::new(), |mut acc, &x| {
        let prev = acc.last().copied().unwrap_or(first);
        let prev_int = prev.trunc() as i32;
        let curr_int = x.trunc() as i32;
        if prev_int != curr_int {
            acc.push(x);
        }
        acc
    });

    iter::once(first)
        .chain(filtered_middle)
        .chain(std::iter::once(last))
        .collect()
}

fn y_axis_for(kind: &MetricKind, bounds: [f64; 2]) -> Axis<'_> {
    let [min_y, max_y] = bounds;
    let axis: Axis<'_> = Axis::default()
        .style(Style::default().fg(Color::Gray))
        .bounds(bounds);
    match kind {
        MetricKind::Bytes => {
            let y_ticks = (0..=2)
                .map(|i| {
                    if i == 0 {
                        0.0
                    } else {
                        min_y + (i as f64) * (max_y - min_y) / 4.0
                    }
                })
                .collect::<Vec<_>>();
            let y_labels = adjust_vec(y_ticks)
                .iter()
                .map(|i| Span::from(format_mib_label(*i)))
                .collect::<Vec<Span>>();

            axis.labels(y_labels)
        }
        MetricKind::Percentage => {
            let y_ticks = vec![Span::from("0 %"), Span::from(format!("{:.1} %", max_y))];

            axis.labels(y_ticks)
        }
    }
}

impl MetricsScreen {
    fn render_single_metric_chart(
        &self,
        frame: &mut Frame,
        area: Rect,
        config: ChartDatasetConfig,
        title: &str,
        kind: &MetricKind,
    ) {
        let Some(metric) = config.data else { return };

        let (x_bounds, y_bounds) = metric.time_series_smoothed.get_bounds();
        let padded_y_bounds = get_padded_y_bounds(y_bounds);

        let data_slice = metric.time_series_smoothed.data();
        let dataset = Dataset::default()
            .name(config.label)
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(config.color))
            .data(&data_slice);

        let chart = Chart::new(vec![dataset])
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Right)
                    .borders(Borders::ALL),
            )
            .x_axis(Axis::default().bounds(x_bounds))
            .y_axis(y_axis_for(kind, padded_y_bounds));
        frame.render_widget(chart, area);
    }

    fn render_dual_metric_chart(
        &self,
        frame: &mut Frame,
        area: Rect,
        config_a: ChartDatasetConfig,
        config_b: ChartDatasetConfig,
        title: &str,
        kind: &MetricKind,
    ) {
        let (Some(metric_a), Some(metric_b)) = (config_a.data, config_b.data) else {
            return;
        };

        // Combine bounds
        let (a_x_bounds, a_y_bounds) = metric_a.time_series_smoothed.get_bounds();
        let (b_x_bounds, b_y_bounds) = metric_b.time_series_smoothed.get_bounds();
        let x_bounds = [
            a_x_bounds[0].min(b_x_bounds[0]),
            a_x_bounds[1].max(b_x_bounds[1]),
        ];
        let y_bounds = [
            a_y_bounds[0].min(b_y_bounds[0]),
            a_y_bounds[1].max(b_y_bounds[1]),
        ];
        let padded_y_bounds = get_padded_y_bounds(y_bounds);

        let data_slice_a = metric_a.time_series_smoothed.data();
        let data_slice_b = metric_b.time_series_smoothed.data();

        let datasets = vec![
            Dataset::default()
                .name(config_a.label)
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(config_a.color))
                .data(&data_slice_a),
            Dataset::default()
                .name(config_b.label)
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(config_b.color))
                .data(&data_slice_b),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Right)
                    .borders(Borders::ALL),
            )
            .x_axis(Axis::default().bounds(x_bounds))
            .y_axis(y_axis_for(kind, padded_y_bounds));
        frame.render_widget(chart, area);
    }
}

impl Screen for MetricsScreen {
    fn kind(&self) -> Kind {
        Kind::Metrics
    }

    fn update(&mut self, _ac: AppContext) -> ScreenAction {
        ScreenAction::None
    }

    /// Render the screen
    fn display(&self, ac: AppContext, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33), // CPU
                Constraint::Percentage(33), // Memory
                Constraint::Percentage(34), // Disk
            ])
            .split(area);

        // --- Chart 1: CPU Usage ---
        let cpu_config = ChartDatasetConfig {
            data: ac.system.metrics.get("process_cpu_live"),
            label: "CPU Cores",
            color: Color::Cyan,
        };
        self.render_single_metric_chart(
            frame,
            chunks[0],
            cpu_config,
            "CPU",
            &MetricKind::Percentage,
        );

        // --- Chart 2: Memory Usage ---
        let mem_config = ChartDatasetConfig {
            data: ac.system.metrics.get("process_memory_live_resident"),
            label: "Memory",
            color: Color::Green,
        };
        self.render_single_metric_chart(
            frame,
            chunks[1],
            mem_config,
            " Memory",
            &MetricKind::Bytes,
        );

        // --- Chart 3: Disk I/O ---
        let read_config = ChartDatasetConfig {
            data: ac.system.metrics.get("process_disk_live_read"),
            label: "Read",
            color: Color::Yellow,
        };
        let write_config = ChartDatasetConfig {
            data: ac.system.metrics.get("process_disk_live_write"),
            label: "Write",
            color: Color::Cyan,
        };
        self.render_dual_metric_chart(
            frame,
            chunks[2],
            read_config,
            write_config,
            " Disk I/O",
            &MetricKind::Bytes,
        );
    }
}
