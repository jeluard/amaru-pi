use crate::{
    button::InputEvent,
    screens::{AppContext, Kind, Screen, ScreenAction},
};
use amaru_doctor::{components::Component, metrics::page::MetricsPageComponent};
use ratatui::{Frame, layout::Rect};

pub struct MetricsScreen {
    component: MetricsPageComponent,
}

impl Default for MetricsScreen {
    fn default() -> Self {
        Self {
            component: MetricsPageComponent::new_with_service(),
        }
    }
}

impl Screen for MetricsScreen {
    fn kind(&self) -> Kind {
        Kind::Metrics
    }

    fn update(&mut self, _ac: AppContext) -> ScreenAction {
        self.component.tick();
        ScreenAction::None
    }

    fn handle_input(&mut self, _event: InputEvent) -> bool {
        false
    }

    fn display(&self, _ac: AppContext, frame: &mut Frame, area: Rect) {
        self.component.render(frame, area);
    }
}
