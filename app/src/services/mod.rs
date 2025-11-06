use crate::app::AppActionComplete;
use tokio::sync::mpsc::Sender;

mod metrics;

pub fn start_all_background_tasks(tx: Sender<AppActionComplete>) {
    metrics::start_metrics_service(tx);
}
