use crate::app::{App, AppAction, AppMsg};
use crate::backends;
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
    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        if let AppAction::Quit = app.update(AppMsg::Tick).await {
            running.store(false, Ordering::SeqCst);
        }

        // TODO multiple events
        if let Ok(event) = input_rx.try_recv()
            && let AppAction::Quit = app.update(AppMsg::Input(event)).await
        {
            running.store(false, Ordering::SeqCst);
        }

        terminal.draw(|frame| {
            app.draw(frame);
        })?;
    }
    terminal.clear()?;

    Ok(())
}
