use crate::screens::Screen;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};
use std::time::Duration;
use tachyonfx::{EffectManager, EffectTimer, Interpolation, fx};

pub struct LogoScreen {
    pub effects: EffectManager<()>,
    triggered: bool,
    // We’ll store the logo area to know where to explode
    logo_area: Option<Rect>,
    splash_duration: Duration,
}

const LOGO: &str = indoc::indoc! {"
    ▄▀▀▄  █▄ ▄█ ▄▀▀▄  █▀▀▄ █  █
    █▀▀█  █ ▀ █ █▀▀█  █▀▀▄ ▀▄▄▀
"};

impl LogoScreen {
    pub fn new(splash_duration: Duration) -> Self {
        let mgr = EffectManager::default();
        Self {
            effects: mgr,
            triggered: false,
            logo_area: None,
            splash_duration,
        }
    }

    fn on_tick(&mut self, elapsed: Duration, frame: &mut Frame) {
        let area = frame.area();
        self.effects
            .process_effects(elapsed.into(), frame.buffer_mut(), area);
    }

    fn trigger_explosion(&mut self) {
        if let Some(area) = self.logo_area {
            // Create an explode effect over that area
            let effect = fx::explode(
                15.0,
                2.0,
                EffectTimer::new(self.splash_duration.into(), Interpolation::Linear),
            ) // duration in ms
            .with_pattern(tachyonfx::pattern::RadialPattern::center())
            .with_filter(tachyonfx::CellFilter::Area(area));
            // optional: chain with fade-out etc
            self.effects.add_effect(effect);
            self.triggered = true;
        }
    }
}

impl Widget for &mut LogoScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw the static logo
        Text::raw(LOGO).render(area, buf);
        // Let effects modify the buffer
        self.effects
            .process_effects(tachyonfx::Duration::from_millis(0), buf, area);
    }
}

impl Screen for LogoScreen {
    fn display(&mut self, elapsed: Duration, frame: &mut Frame) {
        self.on_tick(elapsed, frame);

        let area = frame.area();
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
        self.logo_area = Some(centered);

        frame.render_widget(&mut *self, centered);

        // After first render, you may trigger the explosion
        if !self.triggered {
            self.trigger_explosion();
        }
    }
}
