use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::frame::FrameState;
use crate::network_status::NetworkStatusCache;
use crate::screen_flow::ScreenFlow;
use crate::screens::{AppContext, SystemState};
use crate::systemd::{self, ServiceInfo};
use ratatui::prelude::*;
use std::time::{Duration, Instant};

pub enum AppAction {
    Quit,
    None,
}

pub struct App {
    frame_state: FrameState,
    screen_flow: ScreenFlow,
    connectivity_cache: NetworkStatusCache,
    amaru_status_last_check: Instant,
    amaru_status_interval: Duration,
    system_state: SystemState,
}

impl Default for App {
    fn default() -> Self {
        let default_interval = Duration::from_secs(5);
        let now = Instant::now();
        let connectivity_cache = NetworkStatusCache::new(default_interval);
        let system_state = SystemState {
            amaru_status: ServiceInfo::default(),
            network_status: connectivity_cache.last_result,
        };
        Self {
            frame_state: FrameState::default(),
            screen_flow: ScreenFlow::default(),
            connectivity_cache,
            amaru_status_last_check: now - default_interval,
            amaru_status_interval: default_interval,
            system_state,
        }
    }
}

impl App {
    pub async fn update(&mut self) {
        // Update timing state
        self.frame_state.update();

        // Update network status
        self.system_state.network_status = self.connectivity_cache.get().await;

        // Update amaru status
        if self.amaru_status_last_check.elapsed() >= self.amaru_status_interval {
            self.system_state.amaru_status = tokio::task::spawn_blocking(|| {
                systemd::get_systemd_service_info("amaru").unwrap_or_default()
            })
            .await
            .unwrap_or_default();

            self.amaru_status_last_check = Instant::now();
        }

        let ctx = AppContext {
            frame: &self.frame_state,
            system: &self.system_state,
        };
        self.screen_flow.update(ctx);
    }

    pub fn handle_input(&mut self, event: InputEvent) -> AppAction {
        if !self.screen_flow.handle_input(event)
            && let (ButtonId::B, ButtonPress::Double) = (event.id, event.press_type)
        {
            return AppAction::Quit;
        }
        AppAction::None
    }

    pub fn draw(&self, frame: &mut Frame) {
        let ctx = AppContext {
            frame: &self.frame_state,
            system: &self.system_state,
        };

        self.screen_flow.display(ctx, frame);
    }
}
