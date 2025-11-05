use std::time::{Duration, Instant};

pub struct FrameState {
    pub frame_count: u64,
    pub startup: Instant,
    pub last_loop: Instant,
    pub elapsed_since_startup: Duration,
    pub elapsed_since_last_frame: Duration,
}

impl Default for FrameState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            frame_count: 0,
            startup: now,
            last_loop: now,
            elapsed_since_startup: Duration::ZERO,
            elapsed_since_last_frame: Duration::ZERO,
        }
    }
}

impl FrameState {
    /// Updates the frame count and timers. Call this once per loop.
    pub fn update(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        self.elapsed_since_last_frame = now.duration_since(self.last_loop);
        self.elapsed_since_startup = now.duration_since(self.startup);
        self.last_loop = now;
    }
}
