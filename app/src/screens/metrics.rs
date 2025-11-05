use axum::{
    Router,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use bytes::Bytes;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use prost::Message;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::{self};
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Chart, Dataset};
use std::net::SocketAddr;

use crate::screens::{Kind, State};

pub struct MetricsScreen {
    data1: Vec<(f64, f64)>,
    signal2: SinSignal,
    data2: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl MetricsScreen {
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        let app = Router::new().route(
            "/v1/metrics",
            post(async |_headers: HeaderMap, body: Bytes| {
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

impl MetricsScreen {
    fn on_tick(&mut self) {
        self.data2.drain(0..10);
        self.data2.extend(self.signal2.by_ref().take(10));

        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

impl crate::screens::Screen for MetricsScreen {
    fn kind(&self) -> Kind {
        Kind::Metrics
    }

    fn update(&mut self, _state: State) {
        self.on_tick();
    }

    fn display(&self, _state: State, frame: &mut Frame, area: Rect) -> bool {
        self.render_animated_chart(frame, area);
        true
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
