use super::{KeyboardAction, KeyboardMode, KeyboardWidget};
use crate::button::{ButtonId, ButtonPress, InputEvent, KeyboardInput};
use crate::keyboard::layout::KEYBOARD_LAYOUT;

impl KeyboardWidget {
    /// Handles button presses and returns an optional action.
    pub fn handle_input(&mut self, event: InputEvent) -> Option<KeyboardAction> {
        let max_row = KEYBOARD_LAYOUT.len() - 1;

        if let Some(button) = event.as_button() {
            match (button.id, button.press_type) {
                // In the keyboard, A/B/X/Y are for nav, AA for key press, BB for backspace
                (ButtonId::A, ButtonPress::Short) => {
                    let max_col = KEYBOARD_LAYOUT[self.cursor.0].len() - 1;
                    if self.cursor.1 < max_col {
                        self.cursor.1 += 1;
                    } else {
                        self.cursor.1 = 0;
                    }
                }
                (ButtonId::B, ButtonPress::Short) => {
                    if self.cursor.1 > 0 {
                        self.cursor.1 -= 1;
                    } else {
                        let max_col = KEYBOARD_LAYOUT[self.cursor.0].len() - 1;
                        self.cursor.1 = max_col;
                    }
                }
                (ButtonId::X, ButtonPress::Short) => {
                    if self.cursor.0 > 0 {
                        self.cursor.0 -= 1;
                        self.clamp_cursor_col();
                    }
                }
                (ButtonId::Y, ButtonPress::Short) => {
                    if self.cursor.0 < max_row {
                        self.cursor.0 += 1;
                        self.clamp_cursor_col();
                    }
                }
                (ButtonId::A, ButtonPress::Double) => return self.press_key(),
                (ButtonId::B, ButtonPress::Double) => return Some(KeyboardAction::Backspace),
                (ButtonId::X, ButtonPress::Double) => {
                    if self.cursor.0 > 1 {
                        self.cursor.0 -= 2;
                        self.clamp_cursor_col();
                    }
                }
                (ButtonId::Y, ButtonPress::Double) => {
                    if self.cursor.0 < max_row - 1 {
                        self.cursor.0 += 2;
                        self.clamp_cursor_col();
                    }
                }
                _ => { /* Ignore other button presses */ }
            }
        }

        if let Some(key) = event.as_key() {
            return self.handle_key_input(key);
        }

        None
    }

    /// Checks if the cursor is at the far-right key of the current row.
    pub fn is_cursor_at_right_edge(&self) -> bool {
        let (row, col) = self.cursor;
        let max_col = KEYBOARD_LAYOUT[row].len() - 1;
        col == max_col
    }

    fn clamp_cursor_col(&mut self) {
        let max_col = KEYBOARD_LAYOUT[self.cursor.0].len() - 1;
        if self.cursor.1 > max_col {
            self.cursor.1 = max_col;
        }
    }

    fn handle_key_input(&mut self, key: KeyboardInput) -> Option<KeyboardAction> {
        let max_row = KEYBOARD_LAYOUT.len() - 1;

        match key {
            KeyboardInput::Char(' ') => Some(KeyboardAction::Space),
            KeyboardInput::Char(ch) if !ch.is_control() => {
                Some(KeyboardAction::KeyPress(ch.to_string()))
            }
            KeyboardInput::Char(_) => None,
            KeyboardInput::Backspace => Some(KeyboardAction::Backspace),
            KeyboardInput::Enter | KeyboardInput::Escape => Some(KeyboardAction::Exit),
            KeyboardInput::Left => {
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
                None
            }
            KeyboardInput::Right => {
                let max_col = KEYBOARD_LAYOUT[self.cursor.0].len() - 1;
                if self.cursor.1 < max_col {
                    self.cursor.1 += 1;
                }
                None
            }
            KeyboardInput::Up => {
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                    self.clamp_cursor_col();
                }
                None
            }
            KeyboardInput::Down => {
                if self.cursor.0 < max_row {
                    self.cursor.0 += 1;
                    self.clamp_cursor_col();
                }
                None
            }
            KeyboardInput::Tab | KeyboardInput::BackTab => None,
        }
    }

    fn press_key(&mut self) -> Option<KeyboardAction> {
        let (row, col) = self.cursor;
        let key = KEYBOARD_LAYOUT[row][col];

        match key {
            "Done" => Some(KeyboardAction::Exit),
            "shift" => {
                self.mode = match self.mode {
                    KeyboardMode::Shift => KeyboardMode::Normal,
                    _ => KeyboardMode::Shift,
                };
                None
            }
            "caps" => {
                self.mode = match self.mode {
                    KeyboardMode::CapsLock => KeyboardMode::Normal,
                    _ => KeyboardMode::CapsLock,
                };
                None
            }
            "[ space ]" => Some(KeyboardAction::Space),
            _ => {
                let is_shifted = matches!(self.mode, KeyboardMode::Shift | KeyboardMode::CapsLock);
                let key_str = self.get_key_display_string(key, is_shifted);

                if matches!(self.mode, KeyboardMode::Shift) {
                    // Reset shift
                    self.mode = KeyboardMode::Normal;
                }
                Some(KeyboardAction::KeyPress(key_str))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::button::{InputEvent, KeyboardInput};

    #[test]
    fn direct_character_input_bypasses_virtual_key_selection() {
        let mut keyboard = KeyboardWidget::default();

        assert_eq!(
            keyboard.handle_input(InputEvent::key(KeyboardInput::Char('a'))),
            Some(KeyboardAction::KeyPress("a".into()))
        );
        assert_eq!(
            keyboard.handle_input(InputEvent::key(KeyboardInput::Char(' '))),
            Some(KeyboardAction::Space)
        );
    }

    #[test]
    fn escape_exits_keyboard_focus() {
        let mut keyboard = KeyboardWidget::default();

        assert_eq!(
            keyboard.handle_input(InputEvent::key(KeyboardInput::Escape)),
            Some(KeyboardAction::Exit)
        );
    }
}
