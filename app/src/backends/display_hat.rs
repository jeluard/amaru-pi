use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::{Builder, Display, NoResetPin};
use mousefood::prelude::Rgb565;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use rppal::gpio::{Gpio, InputPin, OutputPin};
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

pub struct GpioPins {
    pub button_a: InputPin,
    pub button_b: InputPin,
    pub button_x: InputPin,
    pub button_y: InputPin,
}

pub fn create_backend() -> (
    EmbeddedBackend<'static, Display<EbSpi, ST7789, NoResetPin>, Rgb565>,
    GpioPins,
) {
    let gpio = Gpio::new().unwrap();
    let dc = gpio.get(SPI_DC).unwrap().into_output();
    let mut backlight = gpio.get(BACKLIGHT).unwrap().into_output();
    backlight.set_high();

    let button_a = gpio.get(BUTTON_A).unwrap().into_input_pullup();
    let button_b = gpio.get(BUTTON_B).unwrap().into_input_pullup();
    let button_x = gpio.get(BUTTON_X).unwrap().into_input_pullup();
    let button_y = gpio.get(BUTTON_Y).unwrap().into_input_pullup();

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
        flush_callback: Box::new(move |_display: &mut Display<EbSpi, ST7789, NoResetPin>| {}),
        ..Default::default()
    };

    let backend = EmbeddedBackend::new(Box::leak(Box::new(display)), backend_config);
    let pins = GpioPins {
        button_a,
        button_b,
        button_x,
        button_y,
    };

    (backend, pins)
}
