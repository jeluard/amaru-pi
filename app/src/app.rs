use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::frame::FrameState;
use crate::network_status::NetworkStatusCache;
use crate::screen_flow::ScreenFlow;
use crate::screens::State;
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
    amaru_status: ServiceInfo,
    amaru_status_last_check: Instant,
    amaru_status_interval: Duration,
}

impl Default for App {
    fn default() -> Self {
        let default_interval = Duration::from_secs(5);
        let now = Instant::now();
        Self {
            frame_state: FrameState::default(),
            screen_flow: ScreenFlow::default(),
            connectivity_cache: NetworkStatusCache::new(default_interval),
            amaru_status: ServiceInfo::default(),
            amaru_status_last_check: now - default_interval,
            amaru_status_interval: default_interval,
        }
    }
}

impl App {
    pub async fn update(&mut self) {
        // Update timing state
        self.frame_state.update();

        // Update network status
        self.connectivity_cache.get().await;

        // Update amaru status if interval has passed
        if self.amaru_status_last_check.elapsed() >= self.amaru_status_interval {
            self.amaru_status = tokio::task::spawn_blocking(|| {
                systemd::get_systemd_service_info("amaru").unwrap_or_default()
            })
            .await
            .unwrap_or_default();

            self.amaru_status_last_check = Instant::now();
        }
    }

    pub fn handle_input(&mut self, event: InputEvent) -> AppAction {
        if !self.screen_flow.handle_input(event)
            && let (ButtonId::B, ButtonPress::Double) = (event.id, event.press_type)
        {
            return AppAction::Quit;
        }
        AppAction::None
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let state = State::new(
            self.frame_state.frame_count,
            self.frame_state.elapsed_since_last_frame,
            self.frame_state.elapsed_since_startup,
            self.amaru_status.clone(),
            self.connectivity_cache.last_result,
        );

        self.screen_flow.display(state, frame);
    }
}
