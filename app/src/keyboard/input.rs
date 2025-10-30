use super::{KeyboardAction, KeyboardMode, KeyboardWidget};
use crate::keyboard::layout::KEYBOARD_LAYOUT;
use amaru_doctor::model::button::{ButtonId, ButtonPress, InputEvent};

impl KeyboardWidget {
    /// Handles button presses and returns an optional action.
    pub fn handle_input(&mut self, event: InputEvent) -> Option<KeyboardAction> {
        let max_row = KEYBOARD_LAYOUT.len() - 1;

        match (event.id, event.press_type) {
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
                    // The cursor is at col 0, wrap around
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
            _ => { /* Ignore other presses */ }
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
