//! Canonical shortcut representation independent of X11 or Wayland.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Modifier {
    Control,
    Alt,
    Super,
    Shift,
}

impl Modifier {
    fn parse(token: &str) -> Option<Self> {
        match token.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Some(Self::Control),
            "alt" => Some(Self::Alt),
            "super" | "meta" => Some(Self::Super),
            "shift" => Some(Self::Shift),
            _ => None,
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Control => "Ctrl",
            Self::Alt => "Alt",
            Self::Super => "Super",
            Self::Shift => "Shift",
        }
    }

    fn display_rank(self) -> u8 {
        match self {
            Self::Control => 0,
            Self::Alt => 1,
            Self::Super => 2,
            Self::Shift => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(String);

impl Key {
    /// Creates one backend-neutral key name. The core accepts future key names
    /// without embedding a complete X11 keysym table; concrete adapters report
    /// unsupported names when they cannot resolve them.
    pub fn new(value: impl Into<String>) -> Result<Self, TriggerParseError> {
        let value = canonicalize_key(value.into().trim())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyChord {
    modifiers: BTreeSet<Modifier>,
    key: Key,
}

impl KeyChord {
    pub fn new(modifiers: impl IntoIterator<Item = Modifier>, key: Key) -> Self {
        Self {
            modifiers: modifiers.into_iter().collect(),
            key,
        }
    }

    pub fn modifiers(&self) -> impl ExactSizeIterator<Item = Modifier> + '_ {
        self.modifiers.iter().copied()
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn has_modifier(&self, modifier: Modifier) -> bool {
        self.modifiers.contains(&modifier)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Trigger {
    Chord(KeyChord),
    /// Fires only after a modifier is pressed and released without participating
    /// in another key chord. This is how bare `Super` coexists with `Super+F`.
    ModifierTap(Modifier),
}

impl Trigger {
    pub fn parse(value: &str) -> Result<Self, TriggerParseError> {
        value.parse()
    }

    /// F11 is explicitly application-owned in the Nexxus contract.
    pub fn is_bare_f11(&self) -> bool {
        matches!(
            self,
            Self::Chord(chord)
                if chord.modifiers.is_empty() && chord.key.as_str().eq_ignore_ascii_case("F11")
        )
    }
}

impl FromStr for Trigger {
    type Err = TriggerParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() {
            return Err(TriggerParseError::Empty);
        }
        let tokens: Vec<&str> = value.split('+').map(str::trim).collect();
        if tokens.iter().any(|token| token.is_empty()) {
            return Err(TriggerParseError::Malformed(value.to_owned()));
        }

        if tokens.len() == 1 {
            if let Some(modifier) = Modifier::parse(tokens[0]) {
                return Ok(Self::ModifierTap(modifier));
            }
            return Ok(Self::Chord(KeyChord::new([], Key::new(tokens[0])?)));
        }

        let mut modifiers = BTreeSet::new();
        for token in &tokens[..tokens.len() - 1] {
            let modifier = Modifier::parse(token)
                .ok_or_else(|| TriggerParseError::UnknownModifier((*token).to_owned()))?;
            if !modifiers.insert(modifier) {
                return Err(TriggerParseError::DuplicateModifier(
                    modifier.display_name().to_owned(),
                ));
            }
        }

        let final_token = tokens[tokens.len() - 1];
        if Modifier::parse(final_token).is_some() {
            return Err(TriggerParseError::MissingKey(value.to_owned()));
        }
        Ok(Self::Chord(KeyChord::new(
            modifiers,
            Key::new(final_token)?,
        )))
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModifierTap(modifier) => formatter.write_str(modifier.display_name()),
            Self::Chord(chord) => {
                let mut modifiers: Vec<Modifier> = chord.modifiers().collect();
                modifiers.sort_by_key(|modifier| modifier.display_rank());
                for modifier in modifiers {
                    write!(formatter, "{}+", modifier.display_name())?;
                }
                write!(formatter, "{}", chord.key)
            }
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TriggerParseError {
    #[error("shortcut trigger cannot be empty")]
    Empty,
    #[error("malformed shortcut trigger '{0}'")]
    Malformed(String),
    #[error("unknown modifier '{0}'")]
    UnknownModifier(String),
    #[error("modifier '{0}' appears more than once")]
    DuplicateModifier(String),
    #[error("shortcut '{0}' does not contain a non-modifier key")]
    MissingKey(String),
    #[error("invalid key name '{0}'")]
    InvalidKey(String),
}

fn canonicalize_key(value: &str) -> Result<String, TriggerParseError> {
    if value.is_empty()
        || value.len() > 64
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace() || byte == b'+')
    {
        return Err(TriggerParseError::InvalidKey(value.to_owned()));
    }

    if value.len() == 1 {
        let byte = value.as_bytes()[0];
        if byte.is_ascii_alphabetic() {
            return Ok((byte as char).to_ascii_uppercase().to_string());
        }
        if byte.is_ascii_digit() {
            return Ok(value.to_owned());
        }
    }

    let lower = value.to_ascii_lowercase();
    let canonical = match lower.as_str() {
        "tab" => "Tab",
        "esc" | "escape" => "Escape",
        "del" | "delete" => "Delete",
        "print" | "printscreen" => "Print",
        "left" => "Left",
        "right" => "Right",
        "up" => "Up",
        "down" => "Down",
        _ if lower.starts_with('f')
            && lower[1..]
                .parse::<u8>()
                .is_ok_and(|number| (1..=35).contains(&number)) =>
        {
            return Ok(lower.to_ascii_uppercase());
        }
        _ if value.starts_with("XF86") => return Ok(value.to_owned()),
        _ => return Ok(value.to_owned()),
    };
    Ok(canonical.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_approved_chords() {
        let trigger = Trigger::parse("ctrl+alt+t").unwrap();
        assert_eq!(trigger.to_string(), "Ctrl+Alt+T");

        let reverse = Trigger::parse("Super+Shift+Tab").unwrap();
        assert_eq!(reverse.to_string(), "Super+Shift+Tab");
    }

    #[test]
    fn bare_super_is_a_modifier_tap() {
        assert_eq!(
            Trigger::parse("Super").unwrap(),
            Trigger::ModifierTap(Modifier::Super)
        );
    }

    #[test]
    fn identifies_only_bare_f11_as_reserved_for_apps() {
        assert!(Trigger::parse("F11").unwrap().is_bare_f11());
        assert!(!Trigger::parse("Super+F11").unwrap().is_bare_f11());
    }

    #[test]
    fn rejects_duplicate_modifiers() {
        assert!(matches!(
            Trigger::parse("Ctrl+Control+T"),
            Err(TriggerParseError::DuplicateModifier(_))
        ));
    }
}
