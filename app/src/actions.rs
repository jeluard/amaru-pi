use crate::app::{App, AppAction};
use crate::systemd;

pub async fn handle_action(app: &mut App, effect: AppAction) {
    match effect {
        // TODO: These should be in background threads
        AppAction::CheckNetworkStatus => {
            app.system_state.network_status = app.connectivity_cache.get().await;
        }
        AppAction::CheckAmaruStatus => {
            app.system_state.amaru_status = tokio::task::spawn_blocking(|| {
                systemd::get_systemd_service_info("amaru").unwrap_or_default()
            })
            .await
            .unwrap_or_default();
        }
        AppAction::Quit => {}
    }
}
