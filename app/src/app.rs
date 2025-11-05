use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::frame::FrameState;
use crate::network_status::NetworkStatusCache;
use crate::screen_flow::ScreenFlow;
use crate::screens::{AppContext, SystemState};
use crate::systemd::ServiceInfo;
use ratatui::prelude::*;
use std::time::{Duration, Instant};

pub enum AppEvent {
    Tick,
    Input(InputEvent),
}

#[derive(PartialEq, Eq)]
pub enum AppAction {
    CheckNetworkStatus,
    CheckAmaruStatus,
    Quit,
}

pub struct App {
    frame_state: FrameState,
    screen_flow: ScreenFlow,
    pub connectivity_cache: NetworkStatusCache,
    amaru_status_last_check: Instant,
    amaru_status_interval: Duration,
    pub system_state: SystemState,
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
    pub fn update(&mut self, msg: AppEvent) -> Vec<AppAction> {
        match msg {
            AppEvent::Tick => {
                self.frame_state.update();
                if self.amaru_status_last_check.elapsed() >= self.amaru_status_interval {
                    self.amaru_status_last_check = Instant::now();
                    return vec![AppAction::CheckNetworkStatus, AppAction::CheckAmaruStatus];
                }
            }
            AppEvent::Input(event) => {
                if !self.screen_flow.handle_input(event)
                    && let (ButtonId::B, ButtonPress::Double) = (event.id, event.press_type)
                {
                    return vec![AppAction::Quit];
                }
            }
        }

        let ctx = AppContext {
            frame: &self.frame_state,
            system: &self.system_state,
        };
        self.screen_flow.update(ctx);

        Vec::new()
    }

    pub fn draw(&self, frame: &mut Frame) {
        let ctx = AppContext {
            frame: &self.frame_state,
            system: &self.system_state,
        };
        self.screen_flow.display(ctx, frame);
    }
}
