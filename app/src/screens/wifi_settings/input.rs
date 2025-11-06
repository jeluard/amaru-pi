use super::{ActiveField, Focus, WiFiSettingsScreen};
use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::keyboard::{KeyboardAction, KeyboardContext};

impl WiFiSettingsScreen {
    pub fn handle_field_navigation(&mut self, event: InputEvent) -> bool {
        match (event.id, event.press_type) {
            // A cycles backward through the fields
            (ButtonId::A, ButtonPress::Short) => {
                self.active_field = match self.active_field {
                    ActiveField::Ssid => ActiveField::ConnectButton,
                    ActiveField::Password => ActiveField::Ssid,
                    ActiveField::PasswordVisibility => ActiveField::Password,
                    ActiveField::ConnectButton => ActiveField::PasswordVisibility,
                };
            }
            // X cycles forward through the fields
            (ButtonId::X, ButtonPress::Short) => {
                self.active_field = match self.active_field {
                    ActiveField::Ssid => ActiveField::Password,
                    ActiveField::Password => ActiveField::PasswordVisibility,
                    ActiveField::PasswordVisibility => ActiveField::ConnectButton,
                    ActiveField::ConnectButton => ActiveField::Ssid,
                };
            }
            // A double-press activates the current field
            (ButtonId::A, ButtonPress::Double) => match self.active_field {
                ActiveField::Ssid => {
                    self.keyboard.set_context(KeyboardContext::Password);
                    self.focus = Focus::Keyboard;
                }
                ActiveField::Password => {
                    self.keyboard.set_context(KeyboardContext::Normal);
                    self.focus = Focus::Keyboard;
                }
                ActiveField::PasswordVisibility => {
                    // Toggle password visibility
                    self.password_visible = !self.password_visible;
                }
                ActiveField::ConnectButton => {
                    // Trigger the connection popup
                    self.focus = Focus::ConnectingPopup;
                }
            },
            _ => {
                return false; // Input was not handled
            }
        }
        true // Input was handled
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
        // Any short press dismisses the popup
        if let InputEvent {
            press_type: ButtonPress::Short,
            ..
        } = event
        {
            self.focus = Focus::Fields;
            self.popup_dismissed = true;
            true
        } else {
            false
        }
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
}
