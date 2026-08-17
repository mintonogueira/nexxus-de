//! Espelho visual mínimo do estado canônico de workspaces.

use nexxus_workspaces::{WorkspaceEvent, WorkspaceId, WorkspaceManager};

/// Item exibido na barra. Nenhum estado de janela é duplicado aqui.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceBarEntry {
    pub id: WorkspaceId,
    pub name: String,
    pub active: bool,
}

/// Ações observáveis produzidas exclusivamente pela interação da barra.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceBarAction {
    Activate(WorkspaceId),
    OpenWorkspaceSettings,
}

/// Modelo de apresentação da barra. A ordem é exatamente a ordem fornecida pelo
/// Workspace Manager e é preservada durante eventos incrementais.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceBarModel {
    entries: Vec<WorkspaceBarEntry>,
}

impl WorkspaceBarModel {
    /// Constrói um snapshot sem assumir nomes ou quantidade de workspaces.
    pub fn from_manager(manager: &WorkspaceManager) -> Self {
        let active = manager.active_id();
        let entries = manager
            .workspaces()
            .map(|workspace| WorkspaceBarEntry {
                id: workspace.id,
                name: workspace.name.clone(),
                active: workspace.id == active,
            })
            .collect();
        Self { entries }
    }

    pub fn entries(&self) -> &[WorkspaceBarEntry] {
        &self.entries
    }

    pub fn active_id(&self) -> Option<WorkspaceId> {
        self.entries
            .iter()
            .find(|entry| entry.active)
            .map(|entry| entry.id)
    }

    /// Aplica somente eventos que alteram o que é visível na barra. Eventos de
    /// associação de janelas não mudam o conteúdo visual e são ignorados.
    pub fn apply_event(&mut self, event: &WorkspaceEvent) -> bool {
        match event {
            WorkspaceEvent::Created {
                workspace, name, ..
            } => {
                if self.entries.iter().any(|entry| entry.id == *workspace) {
                    return false;
                }
                self.entries.push(WorkspaceBarEntry {
                    id: *workspace,
                    name: name.clone(),
                    active: false,
                });
                true
            }
            WorkspaceEvent::Removed { workspace } => {
                let before = self.entries.len();
                self.entries.retain(|entry| entry.id != *workspace);
                self.entries.len() != before
            }
            WorkspaceEvent::Renamed {
                workspace,
                new_name,
                ..
            } => {
                let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == *workspace)
                else {
                    return false;
                };
                if entry.name == *new_name {
                    return false;
                }
                entry.name.clone_from(new_name);
                true
            }
            WorkspaceEvent::Activated { current, .. } => {
                let mut changed = false;
                for entry in &mut self.entries {
                    let active = entry.id == *current;
                    changed |= entry.active != active;
                    entry.active = active;
                }
                changed
            }
            WorkspaceEvent::WindowMoved { .. } | WorkspaceEvent::WindowForgotten { .. } => false,
        }
    }
}
