use amaru_pi::backends;
use amaru_pi::screens::Screen;
use amaru_pi::screens::color::ColorScreen;
use amaru_pi::screens::exit::ExitScreen;
use amaru_pi::screens::logo::LogoScreen;
use amaru_pi::screens::metrics::MetricsScreen;
use amaru_pi::screens::scan::ScanScreen;
use amaru_pi::screens::tip::TipScreen;
use anyhow::Result;
use ratatui::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, current};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Demos from https://github.com/j-g00da/mousefood-esp32-demo/
// https://github.com/j-g00da/mousefood/tree/main/examples/simulator
// https://github.com/orhun/embedded-ratatui-workshop/blob/main/apps/simulator/src/main.rs

#[derive(PartialEq, Clone, Copy)]
enum CurrentScreen {
    Chart,
    Scan,
    Color,
    Tip,
    Exit,
}
const SCREEN_ORDER: [CurrentScreen; 4] = [
    CurrentScreen::Tip,
    CurrentScreen::Chart,
    CurrentScreen::Color,
    CurrentScreen::Scan,
];

fn next_screen(current: CurrentScreen) -> CurrentScreen {
    // Find the current index in SCREEN_ORDER
    if let Some(idx) = SCREEN_ORDER.iter().position(|&s| s == current) {
        // Return the next item, wrapping around
        let next_idx = (idx + 1) % SCREEN_ORDER.len();
        SCREEN_ORDER[next_idx]
    } else {
        // If not found, return the first one or Exit as a fallback
        SCREEN_ORDER[0]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let splash_duration = Duration::from_millis(5000);
    let mut logo = LogoScreen::new(Duration::from_millis(2000), splash_duration);
    let mut chart_screen = MetricsScreen::default();
    /*let mut chart_screen = Arc::new(Mutex::new(MetricsScreen::default()));
            let value = chart_screen.clone();
            tokio::spawn(async move {
                let value = value.clone();
                async move {
                    let mut value = value.lock().await; // async lock
                    value.start().await;                // works fine!
                }
            });
    */
    let mut tip_screen = TipScreen::default();
    let mut color_screen = ColorScreen::default();
    let mut scan_screen = ScanScreen::default();
    let mut exit_screen = ExitScreen::new();

    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input()?;

    #[cfg(feature = "simulator")]
    let backend = backends::simulator::create_backend();

    let mut terminal = Terminal::new(backend).unwrap();

    let mut current_screen = CurrentScreen::Tip;

    let startup = Instant::now();
    let mut last_frame = Instant::now();
    let running = Arc::new(AtomicBool::new(true));
    let mut count = 0;
    while running.load(Ordering::SeqCst) {
        count += 1;
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();
        let show_first = startup.elapsed() < splash_duration;

        #[cfg(feature = "display_hat")]
        {
            // Non-blocking check for button press
            if let Ok(event) = input_rx.try_recv() {
                // Button press detected

                use amaru_doctor::model::button::{ButtonId, ButtonPress};
                match (event.id, event.press_type) {
                    // A exits
                    (ButtonId::A, _) => {
                        println!("Button A pressed. Exiting...");
                        current_screen = CurrentScreen::Exit;
                    }
                    // B cyles screens
                    (ButtonId::B, ButtonPress::Short) => {
                        println!("Button B pressed. Switching screen...");
                        current_screen = next_screen(current_screen);
                    }
                    // Other buttons
                    (ButtonId::X, _) => {
                        println!("Button X pressed: {:?}", event.press_type);
                    }
                    (ButtonId::Y, _) => {
                        println!("Button Y pressed: {:?}", event.press_type);
                    }
                    // Ignore other press types
                    _ => {}
                }
            }
        }

        /*if count % 50 == 0 {
            current_screen = next_screen(current_screen);
        }*/

        terminal.draw(|frame| {
            if show_first {
                logo.display(elapsed, frame);
            } else {
                match current_screen {
                    CurrentScreen::Tip => tip_screen.display(elapsed, frame),
                    CurrentScreen::Chart => chart_screen.display(elapsed, frame),
                    CurrentScreen::Color => color_screen.display(elapsed, frame),
                    CurrentScreen::Scan => scan_screen.display(elapsed, frame),
                    CurrentScreen::Exit => exit_screen.display(elapsed, frame),
                }
            }
        })?;

        if matches!(current_screen, CurrentScreen::Exit) && exit_screen.is_finished() {
            running.store(false, Ordering::SeqCst);
        }
    }

    Ok(())
}
