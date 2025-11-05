use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::screens::info::InfoScreen;
use crate::screens::logo::LogoScreen;
use crate::screens::logs::LogsScreen;
use crate::screens::metrics::MetricsScreen;
use crate::screens::scan::ScanScreen;
use crate::screens::tip::TipScreen;
use crate::screens::wifi_settings::WiFiSettingsScreen;
use crate::screens::{AppContext, Kind, Screen, ScreenAction};
use crate::systemd::ActiveState;
use crate::top_bar::TopBar;
use crate::wifi::Connectivity;
use ratatui::prelude::*;
use std::collections::HashSet;
use std::env;
use std::time::Duration;

pub struct ScreenFlow {
    screens: Vec<Box<dyn Screen>>,
    order: Vec<Kind>,
    pub current_screen_kind: Kind,
}

fn get_screen_order() -> Vec<Kind> {
    let default = vec![
        Kind::Logo,
        Kind::Tip,
        Kind::Metrics,
        Kind::Logs,
        Kind::Scan,
        Kind::Info,
        Kind::WiFiSettings,
    ];
    env::var("AMARU_PI_SCREENS")
        .ok()
        .map(|var| {
            var.split(',')
                .filter_map(|s| s.trim().parse::<Kind>().ok())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or(default)
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
            Box::new(InfoScreen::default()),
        ];
        let order = get_screen_order();
        let current_screen_kind = order
            .first()
            .copied()
            .expect("There must be at least one element in screens order");
        let kinds: Vec<_> = screens.iter().map(|s| s.kind()).collect();
        let unique_kinds: HashSet<_> = kinds.iter().copied().collect();

        if unique_kinds.len() != kinds.len() {
            panic!("Duplicate screen kind detected");
        }
        if let Some(kind) = order.iter().find(|&&k| !unique_kinds.contains(&k)) {
            panic!("No screen found for kind: {:?}", kind);
        }

        Self {
            screens,
            order,
            current_screen_kind,
        }
    }
}

impl ScreenFlow {
    fn screen(&self, kind: Kind) -> &dyn Screen {
        self.screens
            .iter()
            .find(|s| s.kind() == kind)
            .map(|s| &**s)
            .unwrap_or_else(|| panic!("Screen with given kind not found: {}", kind))
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
        let next_idx: usize = (idx + 1) % self.order.len();
        self.order[next_idx]
    }

    /// Get the previous Kind, wraps around
    fn previous_kind(&self, kind: Kind) -> Kind {
        let idx = self
            .order
            .iter()
            .position(|&k| k == kind)
            .expect("Kind not in order");
        let prev_idx = (idx + self.order.len() - 1) % self.order.len();
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
                (ButtonId::Y, ButtonPress::Short) => {
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

    pub fn update(&mut self, ctx: AppContext) -> ScreenAction {
        let action = self.screen_mut(self.current_screen_kind).update(ctx);
        match action {
            ScreenAction::NextScreen => {
                self.update_screen(self.next_kind(self.current_screen_kind));
                ScreenAction::None
            }
            _ => action,
        }
    }

    pub fn display(&self, ctx: AppContext, frame: &mut Frame) {
        let [top_area, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(frame.area());

        let amaru_status_color = match ctx.system.amaru_status.active_state {
            ActiveState::Active => Color::Green,
            ActiveState::Failed => Color::Red,
            _ => Color::Yellow,
        };
        let network_status_color = match ctx.system.network_status.connectivity {
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

        self.screen(self.current_screen_kind)
            .display(ctx, frame, body);
    }
}
