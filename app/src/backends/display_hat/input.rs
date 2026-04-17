use crate::button::{Button, ButtonId, InputEvent};
use anyhow::Result;
use rppal::gpio::InputPin;
use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use super::tty_keyboard;

pub struct InputHandler;

impl InputHandler {
    /// Spawns a dedicated thread to poll GPIO pins and sends events back.
    pub fn spawn(pins: HashMap<ButtonId, InputPin>) -> Result<mpsc::Receiver<InputEvent>> {
        let (tx, rx) = mpsc::channel();

        Self::spawn_gpio_thread(pins, tx.clone());
        tty_keyboard::spawn(tx)?;

        Ok(rx)
    }

    fn spawn_gpio_thread(pins: HashMap<ButtonId, InputPin>, tx: Sender<InputEvent>) {
        let mut buttons: HashMap<ButtonId, Button> =
            pins.keys().map(|id| (*id, Button::default())).collect();

        thread::spawn(move || {
            'poll: loop {
                for (id, button_state) in &mut buttons {
                    let is_low = pins.get(id).unwrap().is_low();

                    if let Some(press_type) = button_state.update(is_low) {
                        if tx.send(InputEvent::button(*id, press_type)).is_err() {
                            break 'poll;
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
