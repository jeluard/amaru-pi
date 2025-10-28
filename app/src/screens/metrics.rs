use amaru_doctor::prometheus::model::Timestamp;
use amaru_doctor::prometheus::service::{MetricsPoller, MetricsPollerHandle};
use amaru_kernel::cbor::decode::info;

use axum::{
    Router,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
};
use bytes::Bytes;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use prost::Message;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::{self, Marker};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, LegendPosition};
use std::net::SocketAddr;
use std::time::Duration;

pub struct MetricsScreen {
    signal1: SinSignal,
    data1: Vec<(f64, f64)>,
    signal2: SinSignal,
    data2: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl MetricsScreen {
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        let app = Router::new().route(
            "/v1/metrics",
            post(async |headers: HeaderMap, body: Bytes| {
                match ExportMetricsServiceRequest::decode(body.as_ref()) {
                    Ok(req) => {
                        for resource_metrics in req.resource_metrics {
                            //println!("Resource metrics: {:?}", resource_metrics.resource);
                            for scope_metrics in resource_metrics.scope_metrics {
                                for metric in scope_metrics.metrics {
                                    println!("Metric: {} {:?}", metric.name, metric.data);
                                }
                            }
                        }
                        StatusCode::OK
                    }
                    Err(err) => {
                        println!("Failed to decode protobuf: {:?}", err);
                        StatusCode::BAD_REQUEST
                    }
                }
            }),
        );

        let addr = SocketAddr::from(([0, 0, 0, 0], 4318));
        println!("Listening on http://{}", addr);

        axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

        Ok(())
    }
}

impl Default for MetricsScreen {
    fn default() -> Self {
        let mut signal1 = SinSignal::new(0.2, 3.0, 18.0);
        let mut signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let data1 = signal1.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        let data2 = signal2.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        Self {
            signal1,
            data1,
            signal2,
            data2,
            window: [0.0, 20.0],
        }
    }
}

#[derive(Clone)]
struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    const fn new(interval: f64, period: f64, scale: f64) -> Self {
        Self {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

fn create_point_f64(metric: &(f64, Timestamp)) -> (f64, f64) {
    let x_time = metric.1.timestamp_micros() as f64 / 1_000_000.0;
    let y_value = metric.0;
    (x_time, y_value)
}

impl MetricsScreen {
    fn on_tick(&mut self, _duration: Duration, _frame: &mut Frame) {
        self.data2.drain(0..10);
        self.data2.extend(self.signal2.by_ref().take(10));

        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

impl crate::screens::Screen for MetricsScreen {
    fn display(&mut self, duration: Duration, frame: &mut Frame) {
        self.on_tick(duration, frame);

        let vertical = Layout::vertical([Constraint::Fill(1); 2]);
        let [top, bottom] = frame.area().layout(&vertical);
        let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Length(29)]);
        let [animated_chart, bar_chart] = top.layout(&horizontal);
        let [line_chart, scatter] = bottom.layout(&Layout::horizontal([Constraint::Fill(1); 2]));

        self.render_animated_chart(frame, frame.area());
        //render_barchart(frame, bar_chart);
        //render_line_chart(frame, line_chart);
        //render_scatter(frame, scatter);
    }
}

impl MetricsScreen {
    fn render_animated_chart(&self, frame: &mut Frame, area: Rect) {
        let x_labels = vec![
            Span::styled(
                format!("{}", self.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", f64::midpoint(self.window[0], self.window[1]))),
            Span::styled(
                format!("{}", self.window[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];
        let datasets = vec![
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.data1),
            Dataset::default()
                .name("data3")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.data2),
        ];

        let chart = Chart::new(datasets)
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels(["-20".bold(), "0".into(), "20".bold()])
                    .bounds([-20.0, 20.0]),
            );

        frame.render_widget(chart, area);
    }
}

fn render_barchart(frame: &mut Frame, bar_chart: Rect) {
    let dataset = Dataset::default()
        .marker(symbols::Marker::HalfBlock)
        .style(Style::new().fg(Color::Blue))
        .graph_type(GraphType::Bar)
        // a bell curve
        .data(&[
            (0., 0.4),
            (10., 2.9),
            (20., 13.5),
            (30., 41.1),
            (40., 80.1),
            (50., 100.0),
            (60., 80.1),
            (70., 41.1),
            (80., 13.5),
            (90., 2.9),
            (100., 0.4),
        ]);

    let chart = Chart::new(vec![dataset])
        .block(Block::bordered().title_top(Line::from("Bar chart").cyan().bold().centered()))
        .x_axis(
            Axis::default()
                .style(Style::default().gray())
                .bounds([0.0, 100.0])
                .labels(["0".bold(), "50".into(), "100.0".bold()]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().gray())
                .bounds([0.0, 100.0])
                .labels(["0".bold(), "50".into(), "100.0".bold()]),
        )
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

    frame.render_widget(chart, bar_chart);
}

fn render_line_chart(frame: &mut Frame, area: Rect) {
    let datasets = vec![
        Dataset::default()
            .name("Line from only 2 points".italic())
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .graph_type(GraphType::Line)
            .data(&[(1., 1.), (4., 4.)]),
    ];

    let chart = Chart::new(datasets)
        .block(Block::bordered().title(Line::from("Line chart").cyan().bold().centered()))
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().gray())
                .bounds([0.0, 5.0])
                .labels(["0".bold(), "2.5".into(), "5.0".bold()]),
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().gray())
                .bounds([0.0, 5.0])
                .labels(["0".bold(), "2.5".into(), "5.0".bold()]),
        )
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

    frame.render_widget(chart, area);
}

// Data from https://ourworldindata.org/space-exploration-satellites
const HEAVY_PAYLOAD_DATA: [(f64, f64); 9] = [
    (1965., 8200.),
    (1967., 5400.),
    (1981., 65400.),
    (1989., 30800.),
    (1997., 10200.),
    (2004., 11600.),
    (2014., 4500.),
    (2016., 7900.),
    (2018., 1500.),
];

const MEDIUM_PAYLOAD_DATA: [(f64, f64); 29] = [
    (1963., 29500.),
    (1964., 30600.),
    (1965., 177_900.),
    (1965., 21000.),
    (1966., 17900.),
    (1966., 8400.),
    (1975., 17500.),
    (1982., 8300.),
    (1985., 5100.),
    (1988., 18300.),
    (1990., 38800.),
    (1990., 9900.),
    (1991., 18700.),
    (1992., 9100.),
    (1994., 10500.),
    (1994., 8500.),
    (1994., 8700.),
    (1997., 6200.),
    (1999., 18000.),
    (1999., 7600.),
    (1999., 8900.),
    (1999., 9600.),
    (2000., 16000.),
    (2001., 10000.),
    (2002., 10400.),
    (2002., 8100.),
    (2010., 2600.),
    (2013., 13600.),
    (2017., 8000.),
];

const SMALL_PAYLOAD_DATA: [(f64, f64); 23] = [
    (1961., 118_500.),
    (1962., 14900.),
    (1975., 21400.),
    (1980., 32800.),
    (1988., 31100.),
    (1990., 41100.),
    (1993., 23600.),
    (1994., 20600.),
    (1994., 34600.),
    (1996., 50600.),
    (1997., 19200.),
    (1997., 45800.),
    (1998., 19100.),
    (2000., 73100.),
    (2003., 11200.),
    (2008., 12600.),
    (2010., 30500.),
    (2012., 20000.),
    (2013., 10600.),
    (2013., 34500.),
    (2015., 10600.),
    (2018., 23100.),
    (2019., 17300.),
];
