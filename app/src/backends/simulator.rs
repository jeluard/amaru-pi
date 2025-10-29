use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay, SimulatorEvent, Window};
use mousefood::{
    EmbeddedBackend, EmbeddedBackendConfig, embedded_graphics::geometry, prelude::Rgb565,
};

// TODO
// Add input handling support
// https://github.com/embedded-graphics/simulator/blob/0afacca1728a0498ee76e690873e8715df077159/examples/input-handling.rs

pub fn create_backend() -> EmbeddedBackend<'static, SimulatorDisplay<Rgb565>, Rgb565> {
    let mut simulator_window = Window::new(
        "Simulator",
        &OutputSettings {
            scale: 2,
            ..Default::default()
        },
    );
    let display = SimulatorDisplay::<Rgb565>::new(geometry::Size::new(320, 240));

    let backend_config: EmbeddedBackendConfig<SimulatorDisplay<Rgb565>, _> =
        EmbeddedBackendConfig {
            // Define how to display newly rendered widgets to the simulator window
            flush_callback: Box::new(move |display| {
                simulator_window.update(display);
                if simulator_window.events().any(|e| e == SimulatorEvent::Quit) {
                    panic!("simulator window closed");
                }
            }),
            ..Default::default()
        };
    EmbeddedBackend::new(Box::leak(Box::new(display)), backend_config)
}
