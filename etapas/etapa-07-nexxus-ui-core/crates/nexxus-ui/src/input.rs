//! Backend-neutral input events consumed by the UI tree.

use crate::geometry::LogicalPoint;

/// Stable identifier allocated by a [`crate::widgets::UiTree`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WidgetId(pub u64);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Escape,
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
}

/// UI input after platform adapters have normalized protocol-specific events.
#[derive(Clone, Debug, PartialEq)]
pub enum UiEvent {
    PointerMove { position: LogicalPoint },
    PointerDown { position: LogicalPoint, button: PointerButton },
    PointerUp { position: LogicalPoint, button: PointerButton },
    Scroll { position: LogicalPoint, delta_x: f32, delta_y: f32 },
    KeyDown { key: Key, modifiers: Modifiers },
    KeyUp { key: Key, modifiers: Modifiers },
    TextInput(String),
}
