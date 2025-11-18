use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    Resize,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for the next event with a timeout
    pub fn next(&self) -> std::io::Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                event::Event::Key(key) => {
                    // Only process key press events to avoid double-triggering
                    if key.kind == event::KeyEventKind::Press {
                        Ok(Event::Key(key))
                    } else {
                        Ok(Event::Tick)
                    }
                }
                event::Event::Resize(_, _) => Ok(Event::Resize),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

/// Check if the key event is Ctrl+C (quit)
#[allow(dead_code)]
pub fn is_quit_key(key: KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

/// Check if the key event is Enter
#[allow(dead_code)]
pub fn is_enter_key(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}

/// Check if the key event is Backspace
#[allow(dead_code)]
pub fn is_backspace_key(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Backspace)
}
