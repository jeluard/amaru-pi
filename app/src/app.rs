use crate::button::InputEvent;
use crate::frame::FrameState;
use crate::modal::Modal;
use crate::network_status::NetworkStatusCache;
use crate::screen_flow::ScreenFlow;
use crate::screens::{AppContext, ScreenAction, SystemState, WifiConnectionStatus};
use crate::systemd::ServiceInfo;
use crate::update::{UpdateManager, UpdateStatus};
use ratatui::prelude::*;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub enum AppEvent {
    Tick,
    Input(InputEvent),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppAction {
    CheckNetworkStatus,
    CheckAmaruStatus,
    ConnectToWifi(String, String),
    Quit,
}

#[derive(Debug)]
pub enum AppActionComplete {
    WifiConnection(WifiConnectionStatus),
}

pub struct App {
    frame_state: FrameState,
    screen_flow: ScreenFlow,
    pub connectivity_cache: NetworkStatusCache,
    amaru_status_last_check: Instant,
    amaru_status_interval: Duration,
    pub system_state: SystemState,
    modal: Modal,
    update_manager: UpdateManager,
    pub action_tx: mpsc::Sender<AppActionComplete>,
    action_rx: mpsc::Receiver<AppActionComplete>,
}

impl Default for App {
    fn default() -> Self {
        let default_interval = Duration::from_secs(5);
        let now = Instant::now();
        let connectivity_cache = NetworkStatusCache::new(default_interval);
        let system_state = SystemState {
            amaru_status: ServiceInfo::default(),
            network_status: connectivity_cache.last_result,
            wifi_connection_status: WifiConnectionStatus::default(),
        };
        let (action_tx, action_rx) = mpsc::channel(100);
        Self {
            frame_state: FrameState::default(),
            screen_flow: ScreenFlow::default(),
            connectivity_cache,
            amaru_status_last_check: now - default_interval,
            amaru_status_interval: default_interval,
            system_state,
            modal: Modal::default(),
            update_manager: UpdateManager::new(Duration::from_secs(5)),
            action_tx,
            action_rx,
        }
    }
}

impl App {
    pub fn update(&mut self, msg: AppEvent) -> Vec<AppAction> {
        let mut actions = Vec::new();

        match msg {
            AppEvent::Tick => {
                self.frame_state.update();

                while let Ok(action_result) = self.action_rx.try_recv() {
                    match action_result {
                        AppActionComplete::WifiConnection(status) => {
                            self.system_state.wifi_connection_status = status;
                        }
                    }
                }

                // Amaru status check
                if self.amaru_status_last_check.elapsed() >= self.amaru_status_interval {
                    self.amaru_status_last_check = Instant::now();
                    actions.push(AppAction::CheckNetworkStatus);
                    actions.push(AppAction::CheckAmaruStatus);
                }

                // Update check if no modal is active
                if !self.modal.is_active()
                    && let UpdateStatus::UpdateReadyToNotify =
                        self.update_manager.check_for_update()
                {
                    self.modal = Modal::UpdatePopup;
                }
            }
            AppEvent::Input(event) => {
                // If a modal is active, it handles the input
                if self.modal.handle_input(event, &mut self.update_manager) {
                    // The modal handled it, don't process further
                    return Vec::new();
                }

                // Modal not active or didn't handle, pass to screen flow
                self.screen_flow.handle_input(event);
            }
        }

        let ctx = AppContext {
            frame: &self.frame_state,
            system: &self.system_state,
        };

        // Let the current screen update and potentially return an action
        let screen_action = self.screen_flow.update(ctx);
        match screen_action {
            ScreenAction::ConnectToWifi(ssid, pw) => {
                actions.push(AppAction::ConnectToWifi(ssid, pw))
            }
            ScreenAction::ResetWifiConnectionStatus => {
                // Handle this sync action immediately
                self.system_state.wifi_connection_status = WifiConnectionStatus::Idle;
            }
            _ => {}
        }

        actions
    }

    pub fn draw(&self, frame: &mut Frame) {
        let ctx = AppContext {
            frame: &self.frame_state,
            system: &self.system_state,
        };
        // Draw the main screen first
        self.screen_flow.display(ctx, frame);

        // Draw the modal on top, if active
        self.modal.draw(frame);
    }
}
