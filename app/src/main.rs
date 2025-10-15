use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
    let backend = simulator::create_backend();

/// BEGIN COPY
 /*
    impl embedded_hal::digital::OutputPin for NoCs {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl embedded_hal::digital::ErrorType for NoCs {
        type Error = core::convert::Infallible;
    }

    
    use embedded_hal_bus::spi::ExclusiveDevice;
    use mipidsi::interface::SpiInterface;
    use mipidsi::models::ST7789;
    use mipidsi::options::{ColorInversion, Orientation, Rotation};
    use mipidsi::{Builder, Display};
    use mousefood::prelude::Rgb565;
    use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
    use rppal::gpio::Gpio;
    use rppal::hal::Delay;
    use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

    pub struct NoCs;

    const W: i32 = 240;
    const H: i32 = 320;

    const BUTTON_A: u8 = 5;
    const BUTTON_B: u8 = 6;
    const BUTTON_X: u8 = 16;
    const BUTTON_Y: u8 = 24;
    const SPI_DC: u8 = 9;
    const BACKLIGHT: u8 = 13;

    const LED_R: u8 = 17;
    const LED_G: u8 = 27;
    const LED_B: u8 = 22;

    let mut button_aa = Button::new();
    let mut button_bb = Button::new();

    let gpio = Gpio::new().unwrap();
    let dc = gpio.get(SPI_DC).unwrap().into_output();
    let mut backlight = gpio.get(BACKLIGHT).unwrap().into_output();
    backlight.set_high();

    let button_a = gpio.get(BUTTON_A).unwrap().into_input_pullup();
    let button_b = gpio.get(BUTTON_B).unwrap().into_input_pullup();
    let _button_x = gpio.get(BUTTON_X).unwrap().into_input_pullup();
    let _button_y = gpio.get(BUTTON_Y).unwrap().into_input_pullup();

    let mut led_r = gpio.get(LED_R).unwrap().into_output();
    let mut led_g = gpio.get(LED_G).unwrap().into_output();
    let mut led_b = gpio.get(LED_B).unwrap().into_output();

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss1, 15_000_000_u32, Mode::Mode0).unwrap();
    let spi_device = ExclusiveDevice::new_no_delay(spi, NoCs).unwrap();
    let buffer = Box::new([0_u8; 512]);

    println!("Display initialized");

    led_r.set_high();
    led_g.set_high();
    led_b.set_high();

    let di = SpiInterface::new(spi_device, dc, Box::leak(buffer));
    let mut delay = Delay::new();

    let mut display: Display<
        SpiInterface<
            '_,
            ExclusiveDevice<Spi, NoCs, embedded_hal_bus::spi::NoDelay>,
            rppal::gpio::OutputPin,
        >,
        ST7789,
        mipidsi::NoResetPin,
    > = Builder::new(ST7789, di)
        .display_size(W as u16, H as u16)
        .orientation(Orientation {
            rotation: Rotation::Deg270,
            mirrored: false,
        })
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();

    let backend_config = EmbeddedBackendConfig {
        // Define how to display newly rendered widgets to the simulator window
        flush_callback: Box::new(
            move |_display: &mut Display<
                SpiInterface<
                    '_,
                    ExclusiveDevice<Spi, NoCs, embedded_hal_bus::spi::NoDelay>,
                    rppal::gpio::OutputPin,
                >,
                ST7789,
                mipidsi::NoResetPin,
            >| {},
        ),
        ..Default::default()
    };
    let backend = EmbeddedBackend::new(Box::leak(Box::new(display)), backend_config);
 */
/// END COPY


    let mut terminal = Terminal::new(backend).unwrap();

    let mut startup = Instant::now();
    let mut last_frame = Instant::now();
    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        let show_first = startup.elapsed() < splash_duration;
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();
        /* if let Some(event) = button_aa.update(!button_a.is_high()) {
            println!("Button A pressed");
            println!("Exiting..");
            running.store(false, Ordering::SeqCst);
        }
        if let Some(event) = button_bb.update(!button_b.is_high()) {
            println!("Button B pressed");
        }*/

        // TODO figure out a better way to handle flow between screens
        terminal.draw(|frame| {
            if show_first {
                logo.display(elapsed, frame);
            } else {
                app.display(elapsed, frame);
            }
        })?;
    }

    Ok(())
}
