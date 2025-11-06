use crate::actions::handle_action;
use crate::app::{App, AppAction, AppEvent};
use crate::{backends, services};
use anyhow::Result;
use ratatui::Terminal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn run() -> Result<()> {
    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input()?;
    #[cfg(feature = "simulator")]
    let (backend, input_rx) = backends::simulator::setup_simulator_and_input();

    let mut terminal = Terminal::new(backend)?;
    let mut app = App::default();
    services::start_all_background_tasks(app.action_tx.clone());
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
