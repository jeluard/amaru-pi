use std::time::{Duration, Instant};

const DEBOUNCE: Duration = Duration::from_millis(50);
const LONG_PRESS: Duration = Duration::from_millis(1000);
const DOUBLE_PRESS: Duration = Duration::from_millis(400);

#[derive(Debug)]
pub enum ButtonPressEvent {
    Short,
    Long,
    Double,
}

pub struct Button {
    pressed: bool,
    last_change: Instant,
    press_start: Option<Instant>,
    long_triggered: bool,

    last_release: Option<Instant>,
    pending_short: bool,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            pressed: false,
            last_change: Instant::now(),
            press_start: None,
            long_triggered: false,
            last_release: None,
            pending_short: false,
        }
    }
}

impl Button {
    /// Call this every loop with current pin state
    pub fn update(&mut self, is_low: bool) -> Option<ButtonPressEvent> {
        let now = Instant::now();

        // Debounce
        if now.duration_since(self.last_change) < DEBOUNCE {
            return None;
        }

        // Pressed
        if is_low && !self.pressed {
            self.pressed = true;
            self.last_change = now;
            self.press_start = Some(now);
            self.long_triggered = false;
        }
        // Released
        else if !is_low && self.pressed {
            self.pressed = false;
            self.last_change = now;

            if let Some(start) = self.press_start
                && !self.long_triggered
                && now.duration_since(start) >= DEBOUNCE
            {
                // candidate short press
                if let Some(last) = self.last_release
                    && now.duration_since(last) <= DOUBLE_PRESS
                {
                    // It's a double press
                    self.pending_short = false;
                    self.last_release = None;
                    return Some(ButtonPressEvent::Double);
                }
                // Maybe a single press, but wait for double window
                self.pending_short = true;
                self.last_release = Some(now);
            }

            self.press_start = None;
        }

        // Long press detection
        if self.pressed
            && !self.long_triggered
            && let Some(start) = self.press_start
            && now.duration_since(start) >= LONG_PRESS
        {
            self.long_triggered = true;
            self.pending_short = false; // cancel short
            return Some(ButtonPressEvent::Long);
        }

        // Resolve pending short if timeout expired
        if self.pending_short
            && let Some(last) = self.last_release
            && now.duration_since(last) > DOUBLE_PRESS
        {
            self.pending_short = false;
            return Some(ButtonPressEvent::Short);
        }

        None
    }
}
