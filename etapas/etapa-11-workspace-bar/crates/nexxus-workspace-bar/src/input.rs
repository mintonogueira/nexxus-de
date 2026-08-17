//! Hit-testing e estado de ponteiro sem dependência de protocolo gráfico.

use nexxus_ui::LogicalPoint;
use nexxus_workspaces::WorkspaceId;

use crate::{WorkspaceBarAction, WorkspaceBarLayout};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceBarTarget {
    Workspace(WorkspaceId),
    Settings,
}

/// Estado transitório de interação. Não existem timers nem animações.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InteractionState {
    pub hovered: Option<WorkspaceBarTarget>,
    pub pressed: Option<WorkspaceBarTarget>,
}

impl InteractionState {
    pub fn hit_test(layout: &WorkspaceBarLayout, point: LogicalPoint) -> Option<WorkspaceBarTarget> {
        for workspace in &layout.workspaces {
            if workspace.rect.contains(point) {
                return Some(WorkspaceBarTarget::Workspace(workspace.id));
            }
        }
        layout.settings.contains(point).then_some(WorkspaceBarTarget::Settings)
    }

    pub fn pointer_move(&mut self, layout: &WorkspaceBarLayout, point: LogicalPoint) -> bool {
        let next = Self::hit_test(layout, point);
        let changed = self.hovered != next;
        self.hovered = next;
        changed
    }

    pub fn pointer_press(&mut self, layout: &WorkspaceBarLayout, point: LogicalPoint) -> bool {
        let next = Self::hit_test(layout, point);
        let changed = self.pressed != next;
        self.pressed = next;
        changed
    }

    /// Uma ação só é emitida quando press e release ocorrem sobre o mesmo alvo,
    /// evitando ativações acidentais durante arraste do ponteiro.
    pub fn pointer_release(
        &mut self,
        layout: &WorkspaceBarLayout,
        point: LogicalPoint,
    ) -> Option<WorkspaceBarAction> {
        let released = Self::hit_test(layout, point);
        let pressed = self.pressed.take();
        if pressed != released {
            return None;
        }
        match released? {
            WorkspaceBarTarget::Workspace(id) => Some(WorkspaceBarAction::Activate(id)),
            WorkspaceBarTarget::Settings => Some(WorkspaceBarAction::OpenWorkspaceSettings),
        }
    }
}
