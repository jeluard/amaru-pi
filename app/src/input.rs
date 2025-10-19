use crate::button::Button;
use amaru_doctor::model::button::{ButtonId, InputEvent};
use anyhow::Result;
use rppal::gpio::InputPin;
use std::{
    collections::HashMap,
    sync::mpsc::{self},
    thread,
    time::Duration,
};

pub struct InputHandler;

impl InputHandler {
    /// Spawns a dedicated thread to poll GPIO pins and sends events back.
    pub fn spawn(pins: HashMap<ButtonId, InputPin>) -> Result<mpsc::Receiver<InputEvent>> {
        let (tx, rx) = mpsc::channel();

        let mut buttons: HashMap<ButtonId, Button> =
            pins.keys().map(|id| (*id, Button::default())).collect();

        thread::spawn(move || {
            loop {
                for (id, button_state) in &mut buttons {
                    let is_low = pins.get(id).unwrap().is_low();

                    if let Some(press_type) = button_state.update(is_low) {
                        let event = InputEvent {
                            id: *id,
                            press_type,
                        };
                        if tx.send(event).is_err() {
                            break; // Main thread has disconnected
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(rx)
    }
}
