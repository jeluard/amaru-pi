use crate::actions::handle_action;
use crate::app::{App, AppAction, AppEvent};
use crate::{backends};
use anyhow::Result;
use ratatui::Terminal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "display_hat")]
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub async fn run() -> Result<()> {
    #[cfg(feature = "display_hat")]
    let _raw_mode_guard = RawModeGuard::new()?;

    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input()?;
    #[cfg(feature = "simulator")]
    let (backend, input_rx) = backends::simulator::setup_simulator_and_input();

    let mut terminal = Terminal::new(backend)?;
    let mut app = App::default();
    let running = Arc::new(AtomicBool::new(true));
    let mut events: Vec<AppEvent> = Vec::with_capacity(4);
    while running.load(Ordering::SeqCst) {
        events.push(AppEvent::Tick);
        while let Ok(event) = input_rx.try_recv() {
            events.push(AppEvent::Input(event));
        }

        for event in events.drain(..) {
            let actions = app.update(event);
            for action in actions {
                if action == AppAction::Quit {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                handle_action(&mut app, action).await
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        terminal.draw(|frame| {
            app.draw(frame);
        })?;
    }
    terminal.clear()?;

    Ok(())
}

#[cfg(feature = "display_hat")]
struct RawModeGuard;

#[cfg(feature = "display_hat")]
impl RawModeGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

#[cfg(feature = "display_hat")]
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        disable_raw_mode().ok();
    }
}
