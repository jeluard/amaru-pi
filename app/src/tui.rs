use crate::backends;
use crate::button::{ButtonId, ButtonPress};
use crate::network_status::NetworkStatusCache;
use crate::screen_flow::ScreenFlow;
use crate::screens::State;
use anyhow::Result;
use ratatui::Terminal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

async fn create_state(
    frame_count: u64,
    elapsed_since_last_frame: Duration,
    elapsed_since_startup: Duration,
    network_status_cache: &mut NetworkStatusCache,
) -> Result<State> {
    let network_status = network_status_cache.get().await;
    Ok(State::new(
        frame_count,
        elapsed_since_last_frame,
        elapsed_since_startup,
        network_status,
    ))
}

pub async fn run() -> Result<()> {
    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input()?;
    #[cfg(feature = "simulator")]
    let (backend, input_rx) = backends::simulator::setup_simulator_and_input();
    let mut terminal = Terminal::new(backend)?;
    let startup = Instant::now();
    let mut frame_count = 0;
    let mut last_loop = Instant::now();

    let mut screen_flow = ScreenFlow::default();
    let mut connectivity_cache = NetworkStatusCache::new(Duration::from_secs(5));
    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        frame_count += 1;
        let elapsed_since_last_frame = last_loop.elapsed();
        last_loop = Instant::now();

        // TODO multiple events
        if let Ok(event) = input_rx.try_recv()
            && !screen_flow.handle_input(event)
            && let (ButtonId::B, ButtonPress::Double) = (event.id, event.press_type)
        {
            running.store(false, Ordering::SeqCst);
        }

        let state = create_state(
            frame_count,
            elapsed_since_last_frame,
            startup.elapsed(),
            &mut connectivity_cache,
        )
        .await?;
        terminal.draw(|frame| {
            screen_flow.display(state, frame);
        })?;
    }
    terminal.clear()?;

    Ok(())
}
