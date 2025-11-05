use crate::button::InputEvent;
use crate::keyboard::KeyboardWidget;
use crate::screens::{AppContext, Kind, Screen, ScreenAction, WifiConnectionStatus};

mod input;
mod render;

#[derive(PartialEq, Eq, Debug)]
pub(super) enum ActiveField {
    Ssid,
    Password,
    PasswordVisibility,
    ConnectButton,
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum Focus {
    Fields,
    Keyboard,
    ConnectingPopup,
}

pub struct WiFiSettingsScreen {
    ssid: String,
    password: String,
    active_field: ActiveField,
    focus: Focus,
    password_visible: bool,
    keyboard: KeyboardWidget,
    popup_dismissed: bool,
}

impl Default for WiFiSettingsScreen {
    fn default() -> Self {
        Self {
            ssid: String::new(),
            password: String::new(),
            active_field: ActiveField::Ssid,
            focus: Focus::Fields,
            password_visible: false,
            keyboard: KeyboardWidget::default(),
            popup_dismissed: false,
        }
    }
}

impl Screen for WiFiSettingsScreen {
    fn kind(&self) -> Kind {
        Kind::WiFiSettings
    }

    fn handle_input(&mut self, event: InputEvent) -> bool {
        match self.focus {
            Focus::Fields => self.handle_field_navigation(event),
            Focus::Keyboard => {
                self.handle_keyboard_input(event);
                true // Keyboard always captures input
            }
            Focus::ConnectingPopup => self.handle_popup_input(event),
        }
    }

    fn update(&mut self, ac: AppContext) -> ScreenAction {
        if self.popup_dismissed {
            self.popup_dismissed = false;
            return ScreenAction::ResetWifiConnectionStatus;
        }

        if self.focus == Focus::ConnectingPopup
            && let WifiConnectionStatus::Idle = ac.system.wifi_connection_status
        {
            // Popup is open, but state is Idle. We need to trigger the connection.
            return ScreenAction::ConnectToWifi(self.ssid.clone(), self.password.clone());
        }
        ScreenAction::None
    }

    fn display(&self, ac: AppContext, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.render_layout(ac, frame, area);
    }
}
