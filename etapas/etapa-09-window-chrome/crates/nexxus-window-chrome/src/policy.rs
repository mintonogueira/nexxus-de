//! Política conservadora CSD/SSD para impedir decoração dupla.

/// Sinais X11 normalizados pelo adapter antes da decisão. O núcleo da política
/// não conhece atoms nem handles X11 e pode ser testado sem servidor gráfico.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DecorationHints {
    pub override_redirect: bool,
    pub gtk_frame_extents: bool,
    pub motif_decorations_disabled: bool,
    pub window_type: WindowType,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WindowType {
    #[default]
    Normal,
    Dialog,
    Utility,
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Splash,
    DropdownMenu,
    PopupMenu,
    Tooltip,
    Notification,
    Combo,
    DragAndDrop,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecorationDecision {
    /// Nexxus desenha e opera a decoração própria.
    ServerSide,
    /// O cliente já fornece sua moldura/titlebar; Nexxus não duplica chrome.
    ClientSide,
    /// A classe de janela não deve receber decoração desktop normal.
    None,
}

/// Resolve CSD/SSD priorizando pedidos explícitos do cliente e tipos especiais.
pub fn decide_decoration(hints: DecorationHints) -> DecorationDecision {
    if hints.override_redirect {
        return DecorationDecision::None;
    }
    if hints.gtk_frame_extents || hints.motif_decorations_disabled {
        return DecorationDecision::ClientSide;
    }
    match hints.window_type {
        WindowType::Normal | WindowType::Dialog | WindowType::Utility | WindowType::Unknown => {
            DecorationDecision::ServerSide
        }
        WindowType::Desktop
        | WindowType::Dock
        | WindowType::Toolbar
        | WindowType::Menu
        | WindowType::Splash
        | WindowType::DropdownMenu
        | WindowType::PopupMenu
        | WindowType::Tooltip
        | WindowType::Notification
        | WindowType::Combo
        | WindowType::DragAndDrop => DecorationDecision::None,
    }
}
