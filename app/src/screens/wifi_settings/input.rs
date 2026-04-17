use super::{ActiveField, Focus, WiFiSettingsScreen};
use crate::button::{ButtonId, ButtonPress, InputEvent, KeyboardInput};
use crate::keyboard::{KeyboardAction, KeyboardContext};

impl WiFiSettingsScreen {
    pub fn handle_field_navigation(&mut self, event: InputEvent) -> bool {
        if let Some(button) = event.as_button() {
            match (button.id, button.press_type) {
                (ButtonId::A, ButtonPress::Short) => self.select_previous_field(),
                (ButtonId::X, ButtonPress::Short) => self.select_next_field(),
                (ButtonId::A, ButtonPress::Double) => self.activate_active_field(),
                _ => return false,
            }
            return true;
        }

        if let Some(key) = event.as_key() {
            match key {
                KeyboardInput::Left | KeyboardInput::Up | KeyboardInput::BackTab => {
                    self.select_previous_field();
                }
                KeyboardInput::Right | KeyboardInput::Down | KeyboardInput::Tab => {
                    self.select_next_field();
                }
                KeyboardInput::Enter => self.activate_active_field(),
                KeyboardInput::Char(' ')
                    if matches!(
                        self.active_field,
                        ActiveField::PasswordVisibility | ActiveField::ConnectButton
                    ) =>
                {
                    self.activate_active_field();
                }
                _ => return false,
            }
            return true;
        }

        false
    }

    pub fn handle_keyboard_input(&mut self, event: InputEvent) {
        if let Some(action) = self.keyboard.handle_input(event) {
            match action {
                KeyboardAction::KeyPress(chars) => self.get_active_string().push_str(&chars),
                KeyboardAction::Space => self.get_active_string().push(' '),
                KeyboardAction::Backspace => {
                    self.get_active_string().pop();
                }
                KeyboardAction::Exit => self.focus = Focus::Fields,
            }
        }
    }

    pub fn handle_popup_input(&mut self, event: InputEvent) -> bool {
        if matches!(
            event,
            InputEvent::Button(crate::button::ButtonEvent {
                press_type: ButtonPress::Short,
                ..
            }) | InputEvent::Key(_)
        ) {
            self.focus = Focus::Fields;
            self.popup_dismissed = true;
            return true;
        }

        false
    }

    /// Helper to get a mutable reference to the currently active input string.
    fn get_active_string(&mut self) -> &mut String {
        match self.active_field {
            ActiveField::Ssid => &mut self.ssid,
            ActiveField::Password => &mut self.password,
            ActiveField::PasswordVisibility | ActiveField::ConnectButton => {
                panic!("No active string for this field")
            }
        }
    }

    fn select_previous_field(&mut self) {
        self.active_field = match self.active_field {
            ActiveField::Ssid => ActiveField::ConnectButton,
            ActiveField::Password => ActiveField::Ssid,
            ActiveField::PasswordVisibility => ActiveField::Password,
            ActiveField::ConnectButton => ActiveField::PasswordVisibility,
        };
    }

    fn select_next_field(&mut self) {
        self.active_field = match self.active_field {
            ActiveField::Ssid => ActiveField::Password,
            ActiveField::Password => ActiveField::PasswordVisibility,
            ActiveField::PasswordVisibility => ActiveField::ConnectButton,
            ActiveField::ConnectButton => ActiveField::Ssid,
        };
    }

    fn activate_active_field(&mut self) {
        match self.active_field {
            ActiveField::Ssid => {
                self.keyboard.set_context(KeyboardContext::Normal);
                self.focus = Focus::Keyboard;
            }
            ActiveField::Password => {
                self.keyboard.set_context(KeyboardContext::Password);
                self.focus = Focus::Keyboard;
            }
            ActiveField::PasswordVisibility => {
                self.password_visible = !self.password_visible;
            }
            ActiveField::ConnectButton => {
                self.focus = Focus::ConnectingPopup;
            }
        }
    }
}
