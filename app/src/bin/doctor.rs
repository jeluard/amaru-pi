use amaru_doctor::{
    app::App, open_chain_db, open_ledger_db, otel::service::OtelCollectorService,
    prometheus::service::MetricsPoller, tui::Tui,
};
use amaru_kernel::network::NetworkName;
#[cfg(feature = "display_hat")]
use amaru_pi::backends::display_hat;
use anyhow::Result;
use std::{path::PathBuf, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    amaru_doctor::logging::init()?;

    let otel_service = OtelCollectorService::new("0.0.0.0:4317");
    let otel_handle = otel_service.start();

    let metrics_service =
        MetricsPoller::new("http://0.0.0.0:8889/metrics", Duration::from_millis(100));
    let metrics_handle = metrics_service.start();

    #[cfg(feature = "display_hat")]
    {
        let (backend, button_events) = display_hat::setup_hardware_and_input()?;

        let mut tui = Tui::new(backend)?;

        let ledger_db_path = Some(PathBuf::from("./ledger.mainnet.db"));
        let chain_db_path = Some(PathBuf::from("./chain.mainnet.db"));

        let ledger_db = open_ledger_db(&ledger_db_path, &NetworkName::Mainnet)?;
        let chain_db = open_chain_db(&chain_db_path, &NetworkName::Mainnet)?;

        let mut app = App::new(
            ledger_db,
            chain_db,
            otel_handle.snapshot,
            metrics_handle.receiver,
            button_events,
            tui.get_frame().area(),
        )?;
        app.run(&mut tui).await?;
    }

    Ok(())
}
