use amaru_pi::backends;
use amaru_pi::button::{ButtonId, ButtonPress};
use amaru_pi::screens::Screen;
use amaru_pi::screens::color::ColorScreen;
use amaru_pi::screens::logo::LogoScreen;
use amaru_pi::screens::scan::ScanScreen;
use amaru_pi::screens::tip::TipScreen;
use amaru_pi::screens::wifi_settings::WiFiSettingsScreen;
use anyhow::Result;
use ratatui::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

// Demos from https://github.com/j-g00da/mousefood-esp32-demo/
// https://github.com/j-g00da/mousefood/tree/main/examples/simulator
// https://github.com/orhun/embedded-ratatui-workshop/blob/main/apps/simulator/src/main.rs

#[derive(PartialEq, Clone, Copy)]
enum CurrentScreen {
    Scan,
    WiFiSettings,
    Tip,
    Color,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting amaru-pi");
    let splash_duration = Duration::from_millis(5000);
    let mut logo = LogoScreen::new(Duration::from_millis(2000), splash_duration);
    let mut tip_screen = TipScreen::default();
    let mut color_screen = ColorScreen::default();
    let mut scan_screen = ScanScreen::default();
    let mut wifi_screen = WiFiSettingsScreen::default();

    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input()?;

    #[cfg(feature = "simulator")]
    let (backend, input_rx) = backends::simulator::setup_simulator_and_input();

    let mut terminal = Terminal::new(backend).unwrap();
    let mut current_screen = CurrentScreen::Tip;

    let startup = Instant::now();
    let mut last_loop = Instant::now();

    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        let elapsed = last_loop.elapsed();
        last_loop = Instant::now();
        let show_first = startup.elapsed() < splash_duration;

        if let Ok(event) = input_rx.try_recv() {
            match current_screen {
                CurrentScreen::WiFiSettings => {
                    // Let the wifi screen handle input outside of the other screens
                    wifi_screen.handle_input(event);

                    if !wifi_screen.is_keyboard_active()
                        && let (ButtonId::A, ButtonPress::Short) = (event.id, event.press_type)
                    {
                        // The wifi screen keyboard isn't active and A was pressed
                        current_screen = CurrentScreen::Tip;
                    }
                }
                _ => {
                    match (event.id, event.press_type) {
                        // A switches screens
                        (ButtonId::A, _) => {
                            current_screen = match current_screen {
                                CurrentScreen::Scan => CurrentScreen::WiFiSettings,
                                // Handled above
                                CurrentScreen::WiFiSettings => unreachable!(),
                                CurrentScreen::Tip => CurrentScreen::Color,
                                CurrentScreen::Color => CurrentScreen::Scan,
                            };
                        }
                        // B exits
                        (ButtonId::B, ButtonPress::Short) => {
                            running.store(false, Ordering::SeqCst);
                        }
                        // Other buttons
                        (ButtonId::X, _) => {}
                        (ButtonId::Y, _) => {}
                        // Ignore other press types
                        _ => {}
                    }
                }
            }
        }

        terminal.draw(|frame| {
            if show_first {
                logo.display(elapsed, frame);
            } else {
                match current_screen {
                    CurrentScreen::WiFiSettings => wifi_screen.display(elapsed, frame),
                    CurrentScreen::Tip => tip_screen.display(elapsed, frame),
                    // CurrentScreen::Chart => chart_screen.display(elapsed, frame),
                    CurrentScreen::Color => color_screen.display(elapsed, frame),
                    CurrentScreen::Scan => scan_screen.display(elapsed, frame),
                }
            }
        })?;
    }

    Ok(())
}
