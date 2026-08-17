//! Backend-neutral input state used for shortcut recognition and capture.

use crate::{Key, KeyChord, Modifier, Trigger};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputKey {
    Modifier(Modifier),
    Key(Key),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputEvent {
    pub key: InputKey,
    pub state: KeyState,
}

impl InputEvent {
    pub const fn modifier(modifier: Modifier, state: KeyState) -> Self {
        Self {
            key: InputKey::Modifier(modifier),
            state,
        }
    }

    pub fn key(key: Key, state: KeyState) -> Self {
        Self {
            key: InputKey::Key(key),
            state,
        }
    }
}

/// Recognizes chords immediately on key press and modifier taps on release.
///
/// A modifier is marked as "used" as soon as it participates in another
/// modifier press or a non-modifier key press. Therefore pressing Super+F never
/// produces a later bare-Super action when Super is released.
#[derive(Default)]
pub struct ShortcutRecognizer {
    pressed_modifiers: BTreeSet<Modifier>,
    used_modifiers: BTreeSet<Modifier>,
}

impl ShortcutRecognizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.pressed_modifiers.clear();
        self.used_modifiers.clear();
    }

    pub fn process(&mut self, event: InputEvent) -> Option<Trigger> {
        match (event.key, event.state) {
            (InputKey::Modifier(modifier), KeyState::Pressed) => {
                if !self.pressed_modifiers.is_empty() {
                    self.used_modifiers
                        .extend(self.pressed_modifiers.iter().copied());
                    self.used_modifiers.insert(modifier);
                }
                self.pressed_modifiers.insert(modifier);
                None
            }
            (InputKey::Modifier(modifier), KeyState::Released) => {
                let was_pressed = self.pressed_modifiers.remove(&modifier);
                let was_used = self.used_modifiers.remove(&modifier);
                if was_pressed && !was_used {
                    Some(Trigger::ModifierTap(modifier))
                } else {
                    None
                }
            }
            (InputKey::Key(key), KeyState::Pressed) => {
                self.used_modifiers
                    .extend(self.pressed_modifiers.iter().copied());
                Some(Trigger::Chord(KeyChord::new(
                    self.pressed_modifiers.iter().copied(),
                    key,
                )))
            }
            (InputKey::Key(_), KeyState::Released) => None,
        }
    }
}

/// Small capture helper used later by Settings. It shares the same recognizer as
/// live dispatch so a captured combination cannot use different semantics.
#[derive(Default)]
pub struct ShortcutCapture {
    recognizer: ShortcutRecognizer,
}

impl ShortcutCapture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.recognizer.reset();
    }

    pub fn process(&mut self, event: InputEvent) -> Option<Trigger> {
        self.recognizer.process(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(value: &str) -> Key {
        Key::new(value).unwrap()
    }

    #[test]
    fn bare_super_fires_only_on_release() {
        let mut recognizer = ShortcutRecognizer::new();
        assert_eq!(
            recognizer.process(InputEvent::modifier(Modifier::Super, KeyState::Pressed)),
            None
        );
        assert_eq!(
            recognizer.process(InputEvent::modifier(Modifier::Super, KeyState::Released)),
            Some(Trigger::ModifierTap(Modifier::Super))
        );
    }

    #[test]
    fn super_chord_suppresses_later_super_tap() {
        let mut recognizer = ShortcutRecognizer::new();
        recognizer.process(InputEvent::modifier(Modifier::Super, KeyState::Pressed));
        assert_eq!(
            recognizer.process(InputEvent::key(key("F"), KeyState::Pressed)),
            Some(Trigger::parse("Super+F").unwrap())
        );
        recognizer.process(InputEvent::key(key("F"), KeyState::Released));
        assert_eq!(
            recognizer.process(InputEvent::modifier(Modifier::Super, KeyState::Released)),
            None
        );
    }

    #[test]
    fn modifier_only_chord_does_not_emit_taps() {
        let mut recognizer = ShortcutRecognizer::new();
        recognizer.process(InputEvent::modifier(Modifier::Super, KeyState::Pressed));
        recognizer.process(InputEvent::modifier(Modifier::Shift, KeyState::Pressed));
        assert_eq!(
            recognizer.process(InputEvent::modifier(Modifier::Shift, KeyState::Released)),
            None
        );
        assert_eq!(
            recognizer.process(InputEvent::modifier(Modifier::Super, KeyState::Released)),
            None
        );
    }
}
