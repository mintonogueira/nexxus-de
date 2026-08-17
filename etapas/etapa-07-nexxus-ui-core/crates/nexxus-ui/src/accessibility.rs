//! Toolkit-independent accessibility metadata.
//!
//! Stage 07 defines the semantic tree consumed by a future AT-SPI bridge; it
//! intentionally does not implement that D-Bus bridge in this stage.

use crate::geometry::LogicalRect;
use crate::input::WidgetId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessibilityRole {
    Window,
    Group,
    Label,
    Button,
    ToggleButton,
    CheckBox,
    TextField,
    List,
    ListItem,
    ScrollArea,
    Menu,
    MenuItem,
    Dialog,
    TabList,
    Tab,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityNode {
    pub id: WidgetId,
    pub role: AccessibilityRole,
    pub label: Option<String>,
    pub value: Option<String>,
    pub bounds: LogicalRect,
    pub focused: bool,
    pub disabled: bool,
    pub checked: Option<bool>,
    pub selected: Option<bool>,
    pub children: Vec<WidgetId>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AccessibilityTree {
    pub root: Option<WidgetId>,
    pub nodes: Vec<AccessibilityNode>,
}
