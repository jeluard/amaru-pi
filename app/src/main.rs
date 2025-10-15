use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "display_hat")]
use amaru_pi::backends::display_hat;
#[cfg(feature = "simulator")]
use amaru_pi::backends::simulator;
use amaru_pi::button::Button;
use amaru_pi::screens::chart::ChartScreen;
use amaru_pi::screens::logo::LogoScreen;
use amaru_pi::screens::scan::ScanScreen;
use amaru_pi::screens::Screen;
use mousefood::EmbeddedBackend;
use ratatui::crossterm::event::{self, Event};
use ratatui::prelude::*;

// Demos from https://github.com/j-g00da/mousefood-esp32-demo/
// https://github.com/j-g00da/mousefood/tree/main/examples/simulator
// https://github.com/orhun/embedded-ratatui-workshop/blob/main/apps/simulator/src/main.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let splash_duration = Duration::from_millis(5000);
    let mut logo = LogoScreen::new(splash_duration);
    let mut app = ScanScreen::new();

    #[cfg(feature = "simulator")]
    let mut backend = simulator::create_backend();

    #[cfg(feature = "display_hat")]
    let mut backend = display_hat::create_backend();

    let mut terminal = Terminal::new(backend).unwrap();

    let mut startup = Instant::now();
    let mut last_frame = Instant::now();
    let running = Arc::new(AtomicBool::new(true));



    use signal_hook::{consts::SIGINT, iterator::Signals};
    let term = Arc::new(AtomicBool::new(false));
    let mut signals = Signals::new([SIGINT])?;

    let value = running.clone();
    std::thread::spawn(move || {
        for sig in signals.forever() {
            if sig == SIGINT {
                value.store(false, Ordering::SeqCst);
            }
        }
    });

    while running.load(Ordering::SeqCst) {
        let show_first = startup.elapsed() < splash_duration;
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        // TODO figure out a better way to handle flow between screens
        terminal.draw(|frame| {
            if show_first {
                logo.display(elapsed, frame);
            } else {
                app.display(elapsed, frame);
            }
        })?;
    }

    terminal.backend_mut().clear()?;
    terminal.backend_mut().flush()?;
    println!("Exiting...");

    Ok(())
}
