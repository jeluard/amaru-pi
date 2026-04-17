use crate::button::{InputEvent, KeyboardInput};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn spawn(tx: Sender<InputEvent>) -> Result<()> {
    thread::Builder::new()
        .name("tty-keyboard-input".into())
        .spawn(move || loop {
            match event::poll(POLL_INTERVAL) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(key_event)) => {
                        if let Some(input_event) = translate_key_event(key_event)
                            && tx.send(input_event).is_err()
                        {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(err) => eprintln!("Failed to read tty keyboard input: {err}"),
                },
                Ok(false) => {}
                Err(err) => eprintln!("Failed to poll tty keyboard input: {err}"),
            }
        })
        .context("failed to spawn tty keyboard input thread")?;

    Ok(())
}

fn translate_key_event(key_event: KeyEvent) -> Option<InputEvent> {
    match key_event.kind {
        KeyEventKind::Press | KeyEventKind::Repeat => {}
        KeyEventKind::Release => return None,
    }

    if key_event
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }

    match key_event.code {
        KeyCode::Enter => Some(InputEvent::key(KeyboardInput::Enter)),
        KeyCode::Backspace => Some(InputEvent::key(KeyboardInput::Backspace)),
        KeyCode::Esc => Some(InputEvent::key(KeyboardInput::Escape)),
        KeyCode::Left => Some(InputEvent::key(KeyboardInput::Left)),
        KeyCode::Right => Some(InputEvent::key(KeyboardInput::Right)),
        KeyCode::Up => Some(InputEvent::key(KeyboardInput::Up)),
        KeyCode::Down => Some(InputEvent::key(KeyboardInput::Down)),
        KeyCode::Tab => Some(InputEvent::key(KeyboardInput::Tab)),
        KeyCode::BackTab => Some(InputEvent::key(KeyboardInput::BackTab)),
        KeyCode::Char(character) => Some(InputEvent::key(KeyboardInput::Char(character))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::translate_key_event;
    use crate::button::{InputEvent, KeyboardInput};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    #[test]
    fn ignores_releases_and_ctrl_sequences() {
        assert_eq!(
            translate_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
                .with_kind(KeyEventKind::Release)),
            None
        );
        assert_eq!(
            translate_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            None
        );
    }

    #[test]
    fn maps_navigation_and_characters() {
        assert_eq!(
            translate_key_event(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(InputEvent::key(KeyboardInput::BackTab))
        );
        assert_eq!(
            translate_key_event(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT)),
            Some(InputEvent::key(KeyboardInput::Char('A')))
        );
    }
}