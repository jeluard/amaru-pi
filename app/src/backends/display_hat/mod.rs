use crate::backends::Backend;
use crate::button::{ButtonId, InputEvent};
use anyhow::Result;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::{Builder, Display, NoResetPin};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use rppal::gpio::{Gpio, OutputPin};
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

pub mod input;

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

pub struct NoCs;
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

type EbSpi = SpiInterface<'static, ExclusiveDevice<Spi, NoCs, NoDelay>, OutputPin>;

/// Initializes the display, GPIO, and the input handler thread.
pub fn setup_hardware_and_input() -> Result<(
    Backend<Display<EbSpi, ST7789, NoResetPin>>,
    Receiver<InputEvent>,
)> {
    println!("Setting up display_hat hardware and input");
    let gpio = Gpio::new()?;
    let dc = gpio.get(SPI_DC)?.into_output();
    let mut backlight = gpio.get(BACKLIGHT)?.into_output();
    backlight.set_high();

    let mut pin_map = HashMap::new();
    pin_map.insert(ButtonId::A, gpio.get(BUTTON_A)?.into_input_pullup());
    pin_map.insert(ButtonId::B, gpio.get(BUTTON_B)?.into_input_pullup());
    pin_map.insert(ButtonId::X, gpio.get(BUTTON_X)?.into_input_pullup());
    pin_map.insert(ButtonId::Y, gpio.get(BUTTON_Y)?.into_input_pullup());
    let input_event_receiver = input::InputHandler::spawn(pin_map)?;

    let mut led_r = gpio.get(LED_R)?.into_output();
    let mut led_g = gpio.get(LED_G)?.into_output();
    let mut led_b = gpio.get(LED_B)?.into_output();
    led_r.set_high();
    led_g.set_high();
    led_b.set_high();

    // Initialize SPI and display
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss1, 15_000_000_u32, Mode::Mode0)?;
    let spi_device = ExclusiveDevice::new_no_delay(spi, NoCs)?;
    let buffer = Box::new([0_u8; 512]);
    let di = SpiInterface::new(spi_device, dc, Box::leak(buffer));
    let mut delay = Delay::new();
    let display: Display<EbSpi, ST7789, NoResetPin> = Builder::new(ST7789, di)
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
        flush_callback: Box::new(move |_display| {}),
        ..Default::default()
    };
    let backend = EmbeddedBackend::new(Box::leak(Box::new(display)), backend_config);

    Ok((backend, input_event_receiver))
}
