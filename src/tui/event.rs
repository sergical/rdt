// Event handling utilities for future expansion
// Currently, event handling is done directly in app.rs

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

/// Key event with context
#[derive(Debug, Clone)]
pub struct AppKeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<KeyEvent> for AppKeyEvent {
    fn from(key: KeyEvent) -> Self {
        Self {
            code: key.code,
            modifiers: key.modifiers,
        }
    }
}

/// Check if a key is a quit key
pub fn is_quit_key(key: &KeyEvent) -> bool {
    matches!(
        key.code,
        KeyCode::Char('q') | KeyCode::Char('Q')
    ) || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}
