use crate::app::{App, AppAction, AppActionComplete};
use crate::screens::{WifiConnectionStatus, WifiModeStatus};
use crate::systemd;
use crate::wifi;
use std::time::Duration;

pub async fn handle_action(app: &mut App, effect: AppAction) {
    match effect {
        // TODO: These should be in background threads
        AppAction::CheckNetworkStatus => {
            app.system_state.network_status = app.connectivity_cache.get().await;
        }
        AppAction::CheckWifiModeStatus => {
            let result = tokio::task::spawn_blocking(wifi::current_operating_mode).await;

            match result {
                Ok(Ok(operating_mode)) => app.sync_wifi_mode_status(operating_mode),
                Ok(Err(error)) => {
                    app.system_state.wifi_mode_status = WifiModeStatus::Fault(error.to_string());
                }
                Err(error) => {
                    app.system_state.wifi_mode_status = WifiModeStatus::Fault(error.to_string());
                }
            }
        }
        AppAction::CheckAmaruStatus => {
            app.system_state.amaru_status = tokio::task::spawn_blocking(|| {
                systemd::get_systemd_service_info("amaru").unwrap_or_default()
            })
            .await
            .unwrap_or_default();
        }
        AppAction::ConnectToWifi(ssid, pw) => {
            app.system_state.wifi_connection_status = WifiConnectionStatus::Connecting;
            let tx = app.action_tx.clone();

            tokio::spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    let _ = wifi::stop_hotspot(Duration::from_secs(10));
                    wifi::set_connection(&ssid, &pw)
                        .and_then(|()| wifi::up_connection(Duration::from_secs(30)))
                })
                .await;

                let final_status = match result {
                    Ok(Ok(())) => WifiConnectionStatus::Success,
                    Ok(Err(e)) => WifiConnectionStatus::Failed(e.to_string()),
                    Err(e) => WifiConnectionStatus::Failed(e.to_string()),
                };

                let _ = tx
                    .send(AppActionComplete::WifiConnection(final_status))
                    .await;
            });
        }
        AppAction::Quit => {}
    }
}
