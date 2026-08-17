//! Initial X11 passive-grab adapter for the backend-neutral shortcut registry.
//!
//! The adapter resolves configured key names against the server keyboard map,
//! discovers Alt/Super modifier slots from the live modifier map and expands
//! grabs across lock-state combinations. Wayland/portal handling is deliberately
//! outside this stage.

use crate::{Key, Modifier, ShortcutRegistry, Trigger};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, GrabMode, ModMask};
use x11rb::rust_connection::RustConnection;

const XK_TAB: u32 = 0xff09;
const XK_ESCAPE: u32 = 0xff1b;
const XK_DELETE: u32 = 0xffff;
const XK_PRINT: u32 = 0xff61;
const XK_LEFT: u32 = 0xff51;
const XK_UP: u32 = 0xff52;
const XK_RIGHT: u32 = 0xff53;
const XK_DOWN: u32 = 0xff54;
const XK_F1: u32 = 0xffbe;
const XK_SHIFT_L: u32 = 0xffe1;
const XK_SHIFT_R: u32 = 0xffe2;
const XK_CONTROL_L: u32 = 0xffe3;
const XK_CONTROL_R: u32 = 0xffe4;
const XK_ALT_L: u32 = 0xffe9;
const XK_ALT_R: u32 = 0xffea;
const XK_SUPER_L: u32 = 0xffeb;
const XK_SUPER_R: u32 = 0xffec;
const XK_NUM_LOCK: u32 = 0xff7f;
const XK_SCROLL_LOCK: u32 = 0xff14;
const XF86_MON_BRIGHTNESS_UP: u32 = 0x1008ff02;
const XF86_MON_BRIGHTNESS_DOWN: u32 = 0x1008ff03;
const XF86_AUDIO_LOWER_VOLUME: u32 = 0x1008ff11;
const XF86_AUDIO_MUTE: u32 = 0x1008ff12;
const XF86_AUDIO_RAISE_VOLUME: u32 = 0x1008ff13;
const XF86_AUDIO_PLAY: u32 = 0x1008ff14;
const XF86_AUDIO_PREV: u32 = 0x1008ff16;
const XF86_AUDIO_NEXT: u32 = 0x1008ff17;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GrabSpec {
    pub trigger: Trigger,
    pub keycode: u8,
    /// Raw core-X11 modifier bits. Keeping the public diagnostic representation
    /// numeric prevents protocol types from leaking into the Shortcuts contract.
    pub modifiers: u16,
}

#[derive(Debug, Error)]
pub enum X11GrabError {
    #[error("X11 shortcut adapter is unavailable: {0}")]
    Unavailable(String),
    #[error("X11 shortcut operation failed: {0}")]
    Protocol(String),
    #[error("X11 keyboard map cannot resolve key '{0}'")]
    UnsupportedKey(String),
    #[error("X11 modifier map does not expose '{0}'")]
    UnsupportedModifier(&'static str),
}

pub struct X11ShortcutGrabs {
    conn: RustConnection,
    root: u32,
    specs: Vec<GrabSpec>,
}

impl X11ShortcutGrabs {
    /// Connects to X11 and installs all currently configured global grabs.
    ///
    /// If one grab fails (for example because another client owns it), every
    /// grab installed by this call is rolled back before the error is returned.
    pub fn install(
        display: Option<&str>,
        registry: &ShortcutRegistry,
    ) -> Result<Self, X11GrabError> {
        let (conn, screen_num) = x11rb::connect(display)
            .map_err(|error| X11GrabError::Unavailable(error.to_string()))?;
        let root = conn
            .setup()
            .roots
            .get(screen_num)
            .ok_or_else(|| X11GrabError::Unavailable("selected X11 screen is missing".into()))?
            .root;

        let keyboard = KeyboardMap::load(&conn)?;
        let modifiers = ModifierMap::load(&conn, &keyboard)?;
        let specs = build_specs(registry, &keyboard, &modifiers)?;

        let mut installed = Vec::with_capacity(specs.len());
        for spec in &specs {
            let cookie = conn
                .grab_key(
                    false,
                    root,
                    ModMask::from(spec.modifiers),
                    spec.keycode,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .map_err(|error| X11GrabError::Protocol(error.to_string()))?;
            if let Err(error) = cookie.check() {
                rollback(&conn, root, &installed);
                return Err(X11GrabError::Protocol(format!(
                    "cannot grab {} (keycode {}, mask {:#x}): {error}",
                    spec.trigger, spec.keycode, spec.modifiers
                )));
            }
            installed.push((spec.keycode, spec.modifiers));
        }
        conn.flush()
            .map_err(|error| X11GrabError::Protocol(error.to_string()))?;

        Ok(Self { conn, root, specs })
    }

    pub fn specs(&self) -> &[GrabSpec] {
        &self.specs
    }

    /// Removes only grabs installed by this adapter instance.
    pub fn uninstall(&mut self) -> Result<(), X11GrabError> {
        for spec in &self.specs {
            self.conn
                .ungrab_key(spec.keycode, self.root, ModMask::from(spec.modifiers))
                .map_err(|error| X11GrabError::Protocol(error.to_string()))?;
        }
        self.conn
            .flush()
            .map_err(|error| X11GrabError::Protocol(error.to_string()))?;
        self.specs.clear();
        Ok(())
    }
}

impl Drop for X11ShortcutGrabs {
    fn drop(&mut self) {
        for spec in &self.specs {
            let _ = self
                .conn
                .ungrab_key(spec.keycode, self.root, ModMask::from(spec.modifiers));
        }
        let _ = self.conn.flush();
    }
}

struct KeyboardMap {
    min_keycode: u8,
    keysyms_per_keycode: usize,
    keysyms: Vec<u32>,
}

impl KeyboardMap {
    fn load(conn: &RustConnection) -> Result<Self, X11GrabError> {
        let setup = conn.setup();
        let min_keycode = setup.min_keycode;
        let count = setup
            .max_keycode
            .checked_sub(min_keycode)
            .and_then(|difference| difference.checked_add(1))
            .ok_or_else(|| X11GrabError::Protocol("invalid X11 keycode range".into()))?;
        let reply = conn
            .get_keyboard_mapping(min_keycode, count)
            .map_err(|error| X11GrabError::Protocol(error.to_string()))?
            .reply()
            .map_err(|error| X11GrabError::Protocol(error.to_string()))?;
        if reply.keysyms_per_keycode == 0 {
            return Err(X11GrabError::Protocol(
                "X11 keyboard map has zero keysyms per keycode".into(),
            ));
        }
        Ok(Self {
            min_keycode,
            keysyms_per_keycode: usize::from(reply.keysyms_per_keycode),
            keysyms: reply.keysyms,
        })
    }

    fn keycodes_for_key(&self, key: &Key) -> Result<Vec<u8>, X11GrabError> {
        let candidates =
            keysyms_for_key(key).ok_or_else(|| X11GrabError::UnsupportedKey(key.to_string()))?;
        let mut result = BTreeSet::new();
        for (offset, symbols) in self.keysyms.chunks(self.keysyms_per_keycode).enumerate() {
            if symbols
                .iter()
                .any(|symbol| candidates.iter().any(|candidate| symbol == candidate))
            {
                let offset = u8::try_from(offset)
                    .map_err(|_| X11GrabError::Protocol("X11 keycode offset overflow".into()))?;
                let keycode = self.min_keycode.checked_add(offset).ok_or_else(|| {
                    X11GrabError::Protocol("X11 keycode calculation overflow".into())
                })?;
                result.insert(keycode);
            }
        }
        Ok(result.into_iter().collect())
    }

    fn keycodes_for_keysyms(&self, candidates: &[u32]) -> Vec<u8> {
        let mut result = BTreeSet::new();
        for (offset, symbols) in self.keysyms.chunks(self.keysyms_per_keycode).enumerate() {
            if symbols
                .iter()
                .any(|symbol| candidates.iter().any(|candidate| symbol == candidate))
            {
                if let Ok(offset) = u8::try_from(offset) {
                    if let Some(keycode) = self.min_keycode.checked_add(offset) {
                        result.insert(keycode);
                    }
                }
            }
        }
        result.into_iter().collect()
    }
}

struct ModifierMap {
    masks: BTreeMap<Modifier, ModMask>,
    ignored_lock_masks: Vec<ModMask>,
}

impl ModifierMap {
    fn load(conn: &RustConnection, keyboard: &KeyboardMap) -> Result<Self, X11GrabError> {
        let reply = conn
            .get_modifier_mapping()
            .map_err(|error| X11GrabError::Protocol(error.to_string()))?
            .reply()
            .map_err(|error| X11GrabError::Protocol(error.to_string()))?;
        let per_modifier = usize::from(reply.keycodes_per_modifier());
        if per_modifier == 0 {
            return Err(X11GrabError::Protocol(
                "X11 modifier map has zero keycodes per modifier".into(),
            ));
        }

        let chunks: Vec<&[u8]> = reply.keycodes.chunks(per_modifier).collect();
        if chunks.len() != 8 {
            return Err(X11GrabError::Protocol(format!(
                "X11 modifier map returned {} modifier groups, expected 8",
                chunks.len()
            )));
        }

        let alt_keys = keyboard.keycodes_for_keysyms(&[XK_ALT_L, XK_ALT_R]);
        let super_keys = keyboard.keycodes_for_keysyms(&[XK_SUPER_L, XK_SUPER_R]);
        let num_lock_keys = keyboard.keycodes_for_keysyms(&[XK_NUM_LOCK]);
        let scroll_lock_keys = keyboard.keycodes_for_keysyms(&[XK_SCROLL_LOCK]);

        let mut masks = BTreeMap::new();
        masks.insert(Modifier::Shift, ModMask::SHIFT);
        masks.insert(Modifier::Control, ModMask::CONTROL);
        masks.insert(
            Modifier::Alt,
            find_modifier_mask(&chunks, &alt_keys)
                .ok_or(X11GrabError::UnsupportedModifier("Alt"))?,
        );
        masks.insert(
            Modifier::Super,
            find_modifier_mask(&chunks, &super_keys)
                .ok_or(X11GrabError::UnsupportedModifier("Super"))?,
        );

        let mut locks = vec![ModMask::LOCK];
        if let Some(mask) = find_modifier_mask(&chunks, &num_lock_keys) {
            locks.push(mask);
        }
        if let Some(mask) = find_modifier_mask(&chunks, &scroll_lock_keys) {
            locks.push(mask);
        }
        locks.sort_by_key(|mask| u16::from(*mask));
        locks.dedup();

        Ok(Self {
            masks,
            ignored_lock_masks: lock_combinations(&locks),
        })
    }

    fn mask_for_modifiers(
        &self,
        modifiers: impl IntoIterator<Item = Modifier>,
    ) -> Result<ModMask, X11GrabError> {
        let mut mask = ModMask::default();
        for modifier in modifiers {
            mask |= *self
                .masks
                .get(&modifier)
                .ok_or(X11GrabError::UnsupportedModifier(match modifier {
                    Modifier::Control => "Control",
                    Modifier::Alt => "Alt",
                    Modifier::Super => "Super",
                    Modifier::Shift => "Shift",
                }))?;
        }
        Ok(mask)
    }
}

fn build_specs(
    registry: &ShortcutRegistry,
    keyboard: &KeyboardMap,
    modifiers: &ModifierMap,
) -> Result<Vec<GrabSpec>, X11GrabError> {
    let modifier_taps: BTreeSet<Modifier> = registry
        .bindings()
        .filter_map(|(trigger, _)| match trigger {
            Trigger::ModifierTap(modifier) => Some(*modifier),
            Trigger::Chord(_) => None,
        })
        .collect();

    let mut raw = BTreeSet::new();
    for (trigger, _) in registry.bindings() {
        match trigger {
            Trigger::ModifierTap(modifier) => {
                let keysyms = modifier_keysyms(*modifier);
                let keycodes = keyboard.keycodes_for_keysyms(keysyms);
                if keycodes.is_empty() {
                    return Err(X11GrabError::UnsupportedModifier(match modifier {
                        Modifier::Control => "Control",
                        Modifier::Alt => "Alt",
                        Modifier::Super => "Super",
                        Modifier::Shift => "Shift",
                    }));
                }
                for keycode in keycodes {
                    for lock_mask in &modifiers.ignored_lock_masks {
                        raw.insert((trigger.clone(), keycode, u16::from(*lock_mask)));
                    }
                }
            }
            Trigger::Chord(chord) => {
                // A passive grab on a bare modifier becomes an active keyboard
                // grab as soon as that modifier is pressed. Chords containing
                // it are therefore observed through that active grab and must
                // not compete with redundant passive chord grabs.
                if chord
                    .modifiers()
                    .any(|modifier| modifier_taps.contains(&modifier))
                {
                    continue;
                }

                let base = modifiers.mask_for_modifiers(chord.modifiers())?;
                for keycode in keyboard.keycodes_for_key(chord.key())? {
                    for lock_mask in &modifiers.ignored_lock_masks {
                        let mask = base | *lock_mask;
                        raw.insert((trigger.clone(), keycode, u16::from(mask)));
                    }
                }
            }
        }
    }

    Ok(raw
        .into_iter()
        .map(|(trigger, keycode, modifiers)| GrabSpec {
            trigger,
            keycode,
            modifiers,
        })
        .collect())
}

fn rollback(conn: &RustConnection, root: u32, installed: &[(u8, u16)]) {
    for (keycode, modifiers) in installed {
        let _ = conn.ungrab_key(*keycode, root, ModMask::from(*modifiers));
    }
    let _ = conn.flush();
}

fn find_modifier_mask(groups: &[&[u8]], candidates: &[u8]) -> Option<ModMask> {
    groups.iter().enumerate().find_map(|(index, group)| {
        if group
            .iter()
            .any(|keycode| *keycode != 0 && candidates.contains(keycode))
        {
            modifier_mask_for_index(index)
        } else {
            None
        }
    })
}

fn modifier_mask_for_index(index: usize) -> Option<ModMask> {
    match index {
        0 => Some(ModMask::SHIFT),
        1 => Some(ModMask::LOCK),
        2 => Some(ModMask::CONTROL),
        3 => Some(ModMask::M1),
        4 => Some(ModMask::M2),
        5 => Some(ModMask::M3),
        6 => Some(ModMask::M4),
        7 => Some(ModMask::M5),
        _ => None,
    }
}

fn lock_combinations(locks: &[ModMask]) -> Vec<ModMask> {
    let mut combinations = vec![ModMask::default()];
    for lock in locks {
        let existing = combinations.clone();
        for base in existing {
            combinations.push(base | *lock);
        }
    }
    combinations.sort_by_key(|mask| u16::from(*mask));
    combinations.dedup();
    combinations
}

fn modifier_keysyms(modifier: Modifier) -> &'static [u32] {
    match modifier {
        Modifier::Shift => &[XK_SHIFT_L, XK_SHIFT_R],
        Modifier::Control => &[XK_CONTROL_L, XK_CONTROL_R],
        Modifier::Alt => &[XK_ALT_L, XK_ALT_R],
        Modifier::Super => &[XK_SUPER_L, XK_SUPER_R],
    }
}

fn keysyms_for_key(key: &Key) -> Option<Vec<u32>> {
    let name = key.as_str();
    if name.len() == 1 {
        let byte = name.as_bytes()[0];
        if byte.is_ascii_alphabetic() {
            let lower = byte.to_ascii_lowercase();
            let upper = byte.to_ascii_uppercase();
            return Some(vec![u32::from(lower), u32::from(upper)]);
        }
        if byte.is_ascii_digit() {
            return Some(vec![u32::from(byte)]);
        }
    }

    let symbol = match name {
        "Tab" => XK_TAB,
        "Escape" => XK_ESCAPE,
        "Delete" => XK_DELETE,
        "Print" => XK_PRINT,
        "Left" => XK_LEFT,
        "Right" => XK_RIGHT,
        "Up" => XK_UP,
        "Down" => XK_DOWN,
        "XF86MonBrightnessUp" => XF86_MON_BRIGHTNESS_UP,
        "XF86MonBrightnessDown" => XF86_MON_BRIGHTNESS_DOWN,
        "XF86AudioLowerVolume" => XF86_AUDIO_LOWER_VOLUME,
        "XF86AudioMute" => XF86_AUDIO_MUTE,
        "XF86AudioRaiseVolume" => XF86_AUDIO_RAISE_VOLUME,
        "XF86AudioPlay" => XF86_AUDIO_PLAY,
        "XF86AudioPrev" => XF86_AUDIO_PREV,
        "XF86AudioNext" => XF86_AUDIO_NEXT,
        _ if name.starts_with('F') => {
            let number = name[1..].parse::<u32>().ok()?;
            if !(1..=35).contains(&number) {
                return None;
            }
            XK_F1 + number - 1
        }
        _ => return None,
    };
    Some(vec![symbol])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_combinations_cover_all_states() {
        let combinations = lock_combinations(&[ModMask::LOCK, ModMask::M2]);
        assert_eq!(combinations.len(), 4);
        assert!(combinations.contains(&ModMask::default()));
        assert!(combinations.contains(&ModMask::LOCK));
        assert!(combinations.contains(&ModMask::M2));
        assert!(combinations.contains(&(ModMask::LOCK | ModMask::M2)));
    }

    #[test]
    fn resolves_every_default_non_modifier_key_name() {
        let registry = ShortcutRegistry::with_defaults();
        for (trigger, _) in registry.bindings() {
            if let Trigger::Chord(chord) = trigger {
                assert!(
                    keysyms_for_key(chord.key()).is_some(),
                    "missing X11 keysym mapping for {}",
                    chord.key()
                );
            }
        }
    }
}
