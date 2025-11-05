use crate::screens::{Kind, Screen, State};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};
use std::cell::RefCell;
use std::time::Duration;
use tachyonfx::{EffectManager, EffectTimer, Interpolation, fx};

pub struct LogoScreen {
    pub effects: RefCell<EffectManager<()>>,
    triggered: bool,
    // We’ll store the logo area to know where to explode
    logo_area: RefCell<Option<Rect>>,
    delay_explosion: Duration,
    splash_duration: Duration,
}

const LOGO: &str = indoc::indoc! {"
    ▄▀▀▄  █▄ ▄█ ▄▀▀▄  █▀▀▄ █  █
    █▀▀█  █ ▀ █ █▀▀█  █▀▀▄ ▀▄▄▀
"};

impl LogoScreen {
    pub fn new(delay_explosion: Duration, splash_duration: Duration) -> Self {
        let mgr = EffectManager::default();
        Self {
            effects: RefCell::new(mgr),
            triggered: false,
            logo_area: RefCell::new(None),
            delay_explosion,
            splash_duration,
        }
    }

    fn on_tick(&self, elapsed: Duration, frame: &mut Frame, area: Rect) {
        self.effects
            .borrow_mut()
            .process_effects(elapsed.into(), frame.buffer_mut(), area);
    }

    fn trigger_explosion(&mut self) {
        if let Some(area) = *self.logo_area.borrow() {
            // Create an explode effect over that area
            let effect = fx::explode(
                15.0,
                2.0,
                EffectTimer::new(self.splash_duration.into(), Interpolation::Linear),
            ) // duration in ms
            .with_pattern(tachyonfx::pattern::RadialPattern::center())
            .with_filter(tachyonfx::CellFilter::Area(area));
            // optional: chain with fade-out etc
            self.effects.borrow_mut().add_effect(effect);
            self.triggered = true;
        }
    }
}

impl Widget for &LogoScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw the static logo
        Text::raw(LOGO).render(area, buf);
        // Let effects modify the buffer
        self.effects
            .borrow_mut()
            .process_effects(tachyonfx::Duration::from_millis(0), buf, area);
    }
}

impl Screen for LogoScreen {
    fn kind(&self) -> Kind {
        Kind::Logo
    }

    fn update(&mut self, state: State) {
        // After first render, you may trigger the explosion
        if !self.triggered && state.elapsed_since_startup >= self.delay_explosion {
            self.trigger_explosion();
        }
    }

    fn display(&self, state: State, frame: &mut Frame, area: Rect) -> bool {
        self.on_tick(state.elapsed_since_last_frame, frame, area);

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Min(3),
                Constraint::Percentage(40),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Min(20),
                Constraint::Percentage(20),
            ])
            .split(vertical[1]);

        let centered = horizontal[1];

        // Save the area so we know where to explode
        self.logo_area.replace(Some(centered));

        frame.render_widget(self, centered);

        state.elapsed_since_startup <= (self.delay_explosion + self.splash_duration)
    }
}
