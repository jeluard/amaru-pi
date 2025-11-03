use crate::backends;
use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::screens::logo::LogoScreen;
use crate::screens::logs::LogsScreen;
use crate::screens::metrics::MetricsScreen;
use crate::screens::scan::ScanScreen;
use crate::screens::tip::TipScreen;
use crate::screens::wifi_settings::WiFiSettingsScreen;
use crate::screens::{Kind, Screen, State};
use crate::wifi::{Connectivity, NetworkState, NetworkStatus, check_network_status};
use anyhow::Result;
use ratatui::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

struct ScreenFlow {
    screens: Vec<Box<dyn Screen>>,
    order: Vec<Kind>,
    pub current_screen_kind: Kind,
}

impl ScreenFlow {
    fn new() -> Self {
        let screens: Vec<Box<dyn Screen>> = vec![
            Box::new(LogoScreen::new(
                Duration::from_millis(2000),
                Duration::from_millis(5000),
            )),
            Box::new(TipScreen::default()),
            Box::new(MetricsScreen::default()),
            Box::new(LogsScreen::default()),
            Box::new(ScanScreen::default()),
            Box::new(WiFiSettingsScreen::default()),
        ];
        let first = Kind::Logo;
        let order = vec![
            first,
            Kind::Tip,
            Kind::Metrics,
            Kind::Logs,
            Kind::Scan,
            Kind::WiFiSettings,
        ];
        let mut seen_kinds = HashSet::new();

        for screen in &screens {
            let kind = screen.kind();
            if !seen_kinds.insert(kind) {
                panic!("Duplicate screen kind detected: {:?}", kind);
            }
        }

        for &kind in &order {
            if !seen_kinds.contains(&kind) {
                panic!("No screen found for kind: {:?}", kind);
            }
        }

        for &kind in &seen_kinds {
            if !order.contains(&kind) {
                panic!("Screen kind {:?} not present in order list", kind);
            }
        }

        Self {
            screens,
            order,
            current_screen_kind: first,
        }
    }

    fn screen_mut(&mut self, kind: Kind) -> &mut dyn Screen {
        self.screens
            .iter_mut()
            .find(|s| s.kind() == kind)
            .map(|s| &mut **s)
            .unwrap_or_else(|| panic!("Screen with given kind not found: {}", kind))
    }

    /// Get the next Kind, wraps around
    fn next_kind(&self, kind: Kind) -> Kind {
        let idx = self
            .order
            .iter()
            .position(|&k| k == kind)
            .expect("Kind not in order");
        let mut next_idx: usize = (idx + 1) % self.order.len();
        if next_idx == 0 {
            next_idx = 1
        }
        self.order[next_idx]
    }

    /// Get the previous Kind, wraps around
    fn previous_kind(&self, kind: Kind) -> Kind {
        let idx = self
            .order
            .iter()
            .position(|&k| k == kind)
            .expect("Kind not in order");
        let mut prev_idx = (idx + self.order.len() - 1) % self.order.len();
        if prev_idx == 0 {
            prev_idx = self.order.len() - 1
        }
        self.order[prev_idx]
    }

    fn update_screen(&mut self, kind: Kind) {
        // Exit the current screen, then enter the next one.
        self.screen_mut(self.current_screen_kind).exit();
        let new = self.screen_mut(kind);
        new.enter();
        self.current_screen_kind = new.kind();
    }

    fn handle_input(&mut self, event: InputEvent) -> bool {
        let handled = {
            let current_screen = self.screen_mut(self.current_screen_kind);
            current_screen.handle_input(event)
        };
        if !handled {
            // Only deal with input if screen hasn't captured it
            match (event.id, event.press_type) {
                (ButtonId::A, ButtonPress::Short) => {
                    self.update_screen(self.next_kind(self.current_screen_kind));
                }
                (ButtonId::B, ButtonPress::Short) => {
                    self.update_screen(self.previous_kind(self.current_screen_kind));
                }
                // Ignore other press types
                _ => (),
            }
        }
        handled
    }

    fn display(&mut self, state: State, frame: &mut Frame) {
        if !self
            .screen_mut(self.current_screen_kind)
            .display(state, frame)
        {
            // Screen display is finished, move to next screen
            self.update_screen(self.next_kind(self.current_screen_kind));
        }
    }
}

fn check_network_status_or_unknown() -> NetworkStatus {
    check_network_status().unwrap_or(NetworkStatus {
        state: NetworkState::Unknown,
        connectivity: Connectivity::Unknown,
    })
}
struct NetworkStatusCache {
    last_check: Instant,
    last_result: NetworkStatus,
    interval: Duration,
}

impl NetworkStatusCache {
    fn new(interval: Duration) -> Self {
        Self {
            last_check: Instant::now() - interval,
            last_result: check_network_status_or_unknown(),
            interval,
        }
    }

    async fn get(&mut self) -> NetworkStatus {
        if self.last_check.elapsed() >= self.interval {
            self.last_result = check_network_status_or_unknown();
            self.last_check = Instant::now();
        }
        self.last_result
    }
}

async fn create_state(
    elapsed: Duration,
    network_status_cache: &mut NetworkStatusCache,
) -> Result<State> {
    let network_status = network_status_cache.get().await;
    Ok(State::new(elapsed, network_status))
}

pub async fn run() -> Result<()> {
    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input();
    #[cfg(feature = "simulator")]
    let (backend, input_rx) = backends::simulator::setup_simulator_and_input();
    let mut terminal = Terminal::new(backend)?;
    let mut last_loop = Instant::now();

    let mut screen_flow = ScreenFlow::new();
    let mut connectivity_cache = NetworkStatusCache::new(Duration::from_secs(5));
    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        let elapsed_since_last_frame = last_loop.elapsed();
        last_loop = Instant::now();

        // TODO multiple events
        if let Ok(event) = input_rx.try_recv()
            && !screen_flow.handle_input(event)
            && let (ButtonId::B, ButtonPress::Double) = (event.id, event.press_type)
        {
            running.store(false, Ordering::SeqCst);
        }

        let state = create_state(elapsed_since_last_frame, &mut connectivity_cache).await?;
        terminal.draw(|frame| {
            screen_flow.display(state, frame);
        })?;
    }
    terminal.clear()?;

    Ok(())
}
