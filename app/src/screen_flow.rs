use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::screens::logo::LogoScreen;
use crate::screens::logs::LogsScreen;
use crate::screens::metrics::MetricsScreen;
use crate::screens::scan::ScanScreen;
use crate::screens::tip::TipScreen;
use crate::screens::wifi_settings::WiFiSettingsScreen;
use crate::screens::{Kind, Screen, State};
use crate::systemd::ActiveState;
use crate::top_bar::TopBar;
use crate::wifi::Connectivity;
use ratatui::prelude::*;
use std::collections::HashSet;
use std::time::Duration;

pub struct ScreenFlow {
    screens: Vec<Box<dyn Screen>>,
    order: Vec<Kind>,
    pub current_screen_kind: Kind,
}

impl Default for ScreenFlow {
    fn default() -> Self {
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
}

impl ScreenFlow {
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

    pub fn handle_input(&mut self, event: InputEvent) -> bool {
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

    pub fn display(&mut self, state: State, frame: &mut Frame) {
        let [top_area, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(frame.area());

        let amaru_status_color = match state.amaru_status.active_state {
            ActiveState::Active => Color::Green,
            ActiveState::Failed => Color::Red,
            _ => Color::Yellow,
        };
        let network_status_color = match state.network_status.connectivity {
            Connectivity::Full => Color::Green,
            Connectivity::None => Color::Red,
            _ => Color::Yellow,
        };
        let top_bar = TopBar {
            title: "Amaru",
            amaru_status_color,
            network_status_color,
            background: Color::Black,
        };

        frame.render_widget(top_bar, top_area);

        if !self
            .screen_mut(self.current_screen_kind)
            .display(state, frame, body)
        {
            // Screen display is finished, move to next screen
            self.update_screen(self.next_kind(self.current_screen_kind));
        }
    }
}
