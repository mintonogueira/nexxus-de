//! Barra superior suspensa de workspaces do Nexxus.
//!
//! A crate mantém o estado visual sincronizado com `nexxus-workspaces`, usa
//! primitivas do `nexxus-ui`, consome o catálogo semântico de Visual Assets e
//! fornece um adapter X11 inicial. A barra nunca possui workspaces próprios: o
//! Workspace Manager da Etapa 05 permanece a fonte canônica de estado.

#![forbid(unsafe_code)]

mod input;
mod layout;
mod model;
mod render;
pub mod x11;

pub use input::{InteractionState, WorkspaceBarTarget};
pub use layout::{MonitorGeometry, WorkspaceBarLayout, WorkspaceBarMetrics, WorkspaceButtonLayout};
pub use model::{WorkspaceBarAction, WorkspaceBarEntry, WorkspaceBarModel};
pub use render::{
    AssetSource, WorkspaceBarPainter, WorkspaceBarRenderError, WorkspaceBarVisualState,
};
