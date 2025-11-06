use crate::app::AppActionComplete;
use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use bytes::Bytes;
use opentelemetry_proto::tonic::{
    collector::metrics::v1::ExportMetricsServiceRequest,
    metrics::v1::{NumberDataPoint, metric::Data as MetricDataProto, number_data_point::Value},
};
use prost::Message;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub fn start_metrics_service(tx: Sender<AppActionComplete>) {
    spawn_metrics_server(tx);
}

fn get_value_from_data_point(dp: &NumberDataPoint) -> f64 {
    match dp.value {
        Some(Value::AsDouble(d)) => d,
        Some(Value::AsInt(i)) => i as f64,
        None => 0.0,
    }
}

/// Parses the OTLP metric data into a simple tuple.
/// This function also filters out metric types we don't care about by returning `None`.
fn parse_metric_data(data: Option<MetricDataProto>) -> Option<(&'static str, f64)> {
    match data {
        Some(MetricDataProto::Gauge(gauge)) => {
            let val = gauge
                .data_points
                .first()
                .map(get_value_from_data_point)
                .unwrap_or(0.0);
            Some(("Gauge", val))
        }
        Some(MetricDataProto::Sum(sum)) => {
            let val = sum
                .data_points
                .first()
                .map(get_value_from_data_point)
                .unwrap_or(0.0);
            Some(("Sum", val))
        }
        // We only care about Gauge and Sum, so return None for everything else
        Some(MetricDataProto::Histogram(_))
        | Some(MetricDataProto::ExponentialHistogram(_))
        | Some(MetricDataProto::Summary(_))
        | None => None,
    }
}

/// Processes a single metric, parsing it and sending it over the channel if valid.
async fn process_metric(
    tx: &Arc<Sender<AppActionComplete>>,
    metric: opentelemetry_proto::tonic::metrics::v1::Metric,
) {
    let name = metric.name;

    // Use our helper to parse and filter the metric data
    if let Some((data_type, value)) = parse_metric_data(metric.data) {
        // Send the name, type, and value to the app
        if tx
            .send(AppActionComplete::MetricReceived(
                name,
                data_type.to_string(),
                value,
            ))
            .await
            .is_err()
        {
            println!("[MetricsServer] Error sending metric to TUI: channel closed.");
        }
    }
}

async fn handle_metrics(
    State(tx): State<Arc<Sender<AppActionComplete>>>,
    _headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    match ExportMetricsServiceRequest::decode(body.as_ref()) {
        Ok(req) => {
            for resource_metrics in req.resource_metrics {
                for scope_metrics in resource_metrics.scope_metrics {
                    for metric in scope_metrics.metrics {
                        // Delegate processing of each metric
                        process_metric(&tx, metric).await;
                    }
                }
            }
            StatusCode::OK
        }
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

/// Spawns the Axum server task.
fn spawn_metrics_server(tx: Sender<AppActionComplete>) {
    let shared_tx = Arc::new(tx);

    tokio::spawn(async move {
        let app = Router::new()
            .route("/v1/metrics", post(handle_metrics))
            .with_state(shared_tx);

        let addr = SocketAddr::from(([0, 0, 0, 0], 4318));
        match axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
            Ok(_) => println!("[MetricsServer] Server exited normally."),
            Err(e) => println!("[MetricsServer] Server exited with error: {}", e),
        }
    });
}
