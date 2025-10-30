use std::collections::HashMap;

mod input;
mod layout;
mod render;

#[derive(Debug)]
pub enum KeyboardAction {
    KeyPress(String),
    Space,
    Backspace,
    Exit,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeyboardContext {
    Normal,
    Password,
}

pub struct KeyboardWidget {
    cursor: (usize, usize),
    mode: KeyboardMode,
    shifted_symbols: HashMap<&'static str, &'static str>,
    context: KeyboardContext,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(super) enum KeyboardMode {
    Normal,
    Shift,
    CapsLock,
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self {
            cursor: (0, 0),
            mode: KeyboardMode::Normal,
            shifted_symbols: layout::get_shifted_symbols(),
            context: KeyboardContext::Normal,
        }
    }
}

impl KeyboardWidget {
    pub fn set_context(&mut self, context: KeyboardContext) {
        self.context = context;
        self.cursor = (0, 0);
    }
}
