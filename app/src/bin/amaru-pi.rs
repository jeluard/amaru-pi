use amaru_pi::backends;
use amaru_pi::screens::Screen;
use amaru_pi::screens::chart::ChartScreen;
use amaru_pi::screens::color::ColorScreen;
use amaru_pi::screens::exit::ExitScreen;
use amaru_pi::screens::logo::LogoScreen;
use anyhow::Result;
use ratatui::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

// Demos from https://github.com/j-g00da/mousefood-esp32-demo/
// https://github.com/j-g00da/mousefood/tree/main/examples/simulator
// https://github.com/orhun/embedded-ratatui-workshop/blob/main/apps/simulator/src/main.rs

enum CurrentScreen {
    Chart,
    Color,
    Exit,
}

#[tokio::main]
async fn main() -> Result<()> {
    let splash_duration = Duration::from_millis(2000);
    let mut logo = LogoScreen::new(splash_duration);
    let mut chart_screen = ChartScreen::default();
    let mut color_screen = ColorScreen::default();
    let mut exit_screen = ExitScreen::new();

    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input()?;

    #[cfg(feature = "simulator")]
    let backend = backends::simulator::create_backend();

    let mut terminal = Terminal::new(backend).unwrap();

    let mut current_screen = CurrentScreen::Chart;

    let startup = Instant::now();
    let mut last_frame = Instant::now();
    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
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
                        current_screen = match current_screen {
                            CurrentScreen::Chart => CurrentScreen::Color,
                            CurrentScreen::Color => CurrentScreen::Chart,
                            CurrentScreen::Exit => CurrentScreen::Exit, // No change
                        };
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

        terminal.draw(|frame| {
            if show_first {
                logo.display(elapsed, frame);
            } else {
                match current_screen {
                    CurrentScreen::Chart => chart_screen.display(elapsed, frame),
                    CurrentScreen::Color => color_screen.display(elapsed, frame),
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
