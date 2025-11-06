use crate::{
    metrics_data::MetricData,
    screens::{AppContext, Kind, Screen, ScreenAction},
};
use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};

pub struct MetricsScreen;

impl Default for MetricsScreen {
    fn default() -> Self {
        Self
    }
}

/// Helper to add padding and generate 5 labels for the Y-axis
fn get_padded_y_bounds(bounds: [f64; 2]) -> ([f64; 2], Vec<Line<'static>>) {
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

    let new_range = max_padded - min_padded;

    // Create 5 labels
    let labels = (0..=4)
        .map(|i| {
            let value = min_padded + (new_range * (i as f64 / 4.0));
            Line::from(format!("{:.1}", value))
        })
        .collect();

    ([min_padded, max_padded], labels)
}

struct ChartDatasetConfig<'a> {
    data: Option<&'a MetricData>,
    label: &'a str,
    color: Color,
}

impl MetricsScreen {
    fn render_single_metric_chart(
        &self,
        frame: &mut Frame,
        area: Rect,
        config: ChartDatasetConfig,
        title: &str,
        y_axis_title: &str,
    ) {
        let Some(metric) = config.data else { return };

        let (x_bounds, y_bounds) = metric.time_series_smoothed.get_bounds();
        let (padded_y_bounds, y_labels) = get_padded_y_bounds(y_bounds);

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
            .y_axis(
                Axis::default()
                    .title(y_axis_title)
                    .style(Style::default().fg(Color::Gray))
                    .bounds(padded_y_bounds)
                    .labels(y_labels),
            );
        frame.render_widget(chart, area);
    }

    fn render_dual_metric_chart(
        &self,
        frame: &mut Frame,
        area: Rect,
        config_a: ChartDatasetConfig,
        config_b: ChartDatasetConfig,
        title: &str,
        y_axis_title: &str,
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
        let (padded_y_bounds, y_labels) = get_padded_y_bounds(y_bounds);

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
            .block(Block::default().title(title).borders(Borders::ALL))
            .x_axis(Axis::default().bounds(x_bounds))
            .y_axis(
                Axis::default()
                    .title(y_axis_title)
                    .style(Style::default().fg(Color::Gray))
                    .bounds(padded_y_bounds)
                    .labels(y_labels),
            );
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
        self.render_single_metric_chart(frame, chunks[0], cpu_config, " CPU Usage ", "Cores");

        // --- Chart 2: Memory Usage ---
        let mem_config = ChartDatasetConfig {
            data: ac.system.metrics.get("process_memory_live_resident"),
            label: "Memory (MB)",
            color: Color::Green,
        };
        self.render_single_metric_chart(frame, chunks[1], mem_config, " Memory (MB)", "MB");

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
            " Disk I/O (B/s) ",
            "B/s",
        );
    }
}
