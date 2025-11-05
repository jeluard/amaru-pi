use crate::app::{App, AppAction};
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
        app.update().await;

        // TODO multiple events
        if let Ok(event) = input_rx.try_recv()
            && let AppAction::Quit = app.handle_input(event)
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
