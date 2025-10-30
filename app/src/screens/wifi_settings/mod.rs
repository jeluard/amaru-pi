use crate::keyboard::KeyboardWidget;

mod input;
mod render;

#[derive(PartialEq, Eq, Debug)]
pub(super) enum ActiveField {
    Ssid,
    Password,
    PasswordVisibility,
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum Focus {
    Fields,
    Keyboard,
}

pub struct WiFiSettingsScreen {
    ssid: String,
    password: String,
    active_field: ActiveField,
    focus: Focus,
    password_visible: bool,
    keyboard: KeyboardWidget,
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
        }
    }
}
