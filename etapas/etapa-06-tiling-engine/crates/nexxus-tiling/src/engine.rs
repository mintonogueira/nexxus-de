//! Stateful assisted-tiling coordinator built on the WM and Workspace contracts.

use crate::layout::fit_slot;
use crate::{LayoutError, LayoutSpec, OutputArea, OutputId, SnapIntent, SnapTarget};
use nexxus_wm::{
    BackendCommandSink, Geometry, PresentationState, WindowId, WindowManager, WindowPlacement,
    WmCommand, WmError,
};
use nexxus_workspaces::{WorkspaceId, WorkspaceManager};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

/// Stable action identifier consumed by the later global Shortcuts Core.
pub const TILE_FIT_ACTION_ID: &str = "nexxus.tiling.tile-fit";

/// Approved default binding descriptor. Global key grabbing/dispatch remains the
/// responsibility of the dedicated Shortcuts Core stage.
pub const DEFAULT_TILE_FIT_SHORTCUT: &str = "Super+T";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TilingAction {
    TileFit,
}

/// Identifies how a tiled window reached its current target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TileTarget {
    LayoutSlot(usize),
    Snap(SnapTarget),
}

/// Runtime assignment used only for deterministic slot selection and diagnostics.
///
/// It binds one window to one output target, never a workspace itself to an
/// output, preserving the Nexxus multi-monitor workspace model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Assignment {
    pub workspace: WorkspaceId,
    pub output: OutputId,
    pub target: TileTarget,
}

/// Geometry and backend-neutral commands produced by one tile operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TilePlan {
    pub window: WindowId,
    pub workspace: WorkspaceId,
    pub output: OutputId,
    pub target: TileTarget,
    pub geometry: Geometry,
    commands: [WmCommand; 2],
}

impl TilePlan {
    pub fn commands(&self) -> &[WmCommand; 2] {
        &self.commands
    }
}

/// Geometry and commands produced when a tiled window returns to floating.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UntilePlan {
    pub window: WindowId,
    pub geometry: Geometry,
    commands: [WmCommand; 2],
}

impl UntilePlan {
    pub fn commands(&self) -> &[WmCommand; 2] {
        &self.commands
    }
}

/// Observable hooks for UI, diagnostics and future integration stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TilingEvent {
    LayoutChanged {
        workspace: WorkspaceId,
    },
    Tiled {
        window: WindowId,
        workspace: WorkspaceId,
        output: OutputId,
        target: TileTarget,
        geometry: Geometry,
    },
    Released {
        window: WindowId,
        geometry: Geometry,
    },
    LayoutChoicesRequested {
        workspace: WorkspaceId,
        output: OutputId,
    },
}

#[derive(Debug, Error)]
pub enum TilingError {
    #[error("window '{0:?}' is not registered in the Window Manager")]
    UnknownWindow(WindowId),
    #[error(
        "window '{window:?}' belongs to workspace {actual:?}, not requested workspace {requested:?}"
    )]
    WorkspaceMismatch {
        window: WindowId,
        requested: WorkspaceId,
        actual: Option<WorkspaceId>,
    },
    #[error(
        "workspace '{workspace:?}' has no free slot on output '{output:?}' for assisted tile-fit"
    )]
    NoAvailableSlot {
        workspace: WorkspaceId,
        output: OutputId,
    },
    #[error("window '{0:?}' is not currently tiled")]
    NotTiled(WindowId),
    #[error(transparent)]
    Layout(#[from] LayoutError),
    #[error(transparent)]
    WindowManager(#[from] WmError),
}

/// Owns per-workspace layouts and runtime tile assignments.
///
/// The default layout is a neutral two-column fallback so `tile-fit` is always
/// actionable before the later Settings UI provides explicit workspace layouts.
pub struct TilingEngine {
    default_layout: LayoutSpec,
    layouts: BTreeMap<WorkspaceId, LayoutSpec>,
    assignments: BTreeMap<WindowId, Assignment>,
    events: VecDeque<TilingEvent>,
}

impl Default for TilingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TilingEngine {
    pub fn new() -> Self {
        Self {
            default_layout: LayoutSpec::balanced_columns(2)
                .expect("static two-column default layout is valid"),
            layouts: BTreeMap::new(),
            assignments: BTreeMap::new(),
            events: VecDeque::new(),
        }
    }

    /// Replaces the fallback used only by workspaces without an explicit layout.
    pub fn set_default_layout(&mut self, layout: LayoutSpec) {
        self.default_layout = layout;
    }

    /// Assigns a layout to one logical workspace without binding that workspace
    /// to any monitor. Existing slot assignments beyond the new layout are
    /// discarded so they cannot become stale.
    pub fn set_layout(&mut self, workspace: WorkspaceId, layout: LayoutSpec) {
        let new_len = layout.len();
        self.layouts.insert(workspace, layout);
        self.assignments.retain(|_, assignment| {
            assignment.workspace != workspace
                || !matches!(
                    assignment.target,
                    TileTarget::LayoutSlot(slot) if slot >= new_len
                )
        });
        self.events
            .push_back(TilingEvent::LayoutChanged { workspace });
    }

    pub fn clear_layout(&mut self, workspace: WorkspaceId) {
        self.layouts.remove(&workspace);
        self.assignments.retain(|_, assignment| {
            assignment.workspace != workspace
                || !matches!(assignment.target, TileTarget::LayoutSlot(_))
        });
        self.events
            .push_back(TilingEvent::LayoutChanged { workspace });
    }

    pub fn layout_for(&self, workspace: WorkspaceId) -> &LayoutSpec {
        self.layouts
            .get(&workspace)
            .unwrap_or(&self.default_layout)
    }

    pub fn assignment(&self, window: WindowId) -> Option<Assignment> {
        self.assignments.get(&window).copied()
    }

    /// Implements the approved `Super+T` semantics for the active workspace.
    ///
    /// Key acquisition is intentionally not duplicated here; the future
    /// Shortcuts Core maps `DEFAULT_TILE_FIT_SHORTCUT` to this action contract.
    pub fn tile_fit_active(
        &mut self,
        workspaces: &WorkspaceManager,
        wm: &mut WindowManager,
        area: OutputArea,
        window: WindowId,
    ) -> Result<TilePlan, TilingError> {
        self.tile_fit(workspaces, wm, workspaces.active_id(), area, window)
    }

    /// Fits one window into the first deterministic free slot for the selected
    /// workspace/output. Existing assignment of the same window is idempotent.
    pub fn tile_fit(
        &mut self,
        workspaces: &WorkspaceManager,
        wm: &mut WindowManager,
        workspace: WorkspaceId,
        area: OutputArea,
        window: WindowId,
    ) -> Result<TilePlan, TilingError> {
        self.validate_membership(workspaces, workspace, window)?;
        let constraints = self.window_constraints(wm, window)?;

        let slot_index = self.select_layout_slot(workspace, area.output, window)?;
        let slot = self
            .layout_for(workspace)
            .slot(slot_index)
            .expect("slot index was selected from the current layout");
        let geometry = fit_slot(area, slot, constraints)?;

        self.apply_tiled_state(wm, window)?;
        let plan = self.make_tile_plan(
            wm,
            window,
            workspace,
            area.output,
            TileTarget::LayoutSlot(slot_index),
            geometry,
        )?;

        self.assignments.insert(
            window,
            Assignment {
                workspace,
                output: area.output,
                target: TileTarget::LayoutSlot(slot_index),
            },
        );
        self.events.push_back(TilingEvent::Tiled {
            window,
            workspace,
            output: area.output,
            target: TileTarget::LayoutSlot(slot_index),
            geometry,
        });
        Ok(plan)
    }

    /// Applies one snap intent generated by `SnapDetector`.
    ///
    /// `ShowLayoutChoices` never changes window state; it only emits the hook
    /// consumed later by the visual overlay.
    pub fn apply_snap(
        &mut self,
        intent: SnapIntent,
        workspaces: &WorkspaceManager,
        wm: &mut WindowManager,
        area: OutputArea,
        window: WindowId,
    ) -> Result<Option<TilePlan>, TilingError> {
        let workspace = workspaces.active_id();
        self.validate_membership(workspaces, workspace, window)?;

        match intent {
            SnapIntent::ShowLayoutChoices => {
                self.events
                    .push_back(TilingEvent::LayoutChoicesRequested {
                        workspace,
                        output: area.output,
                    });
                Ok(None)
            }
            SnapIntent::Tile { target, slot } => {
                let constraints = self.window_constraints(wm, window)?;
                let geometry = fit_slot(area, slot, constraints)?;
                self.apply_tiled_state(wm, window)?;
                let tile_target = TileTarget::Snap(target);
                let plan =
                    self.make_tile_plan(wm, window, workspace, area.output, tile_target, geometry)?;
                self.assignments.insert(
                    window,
                    Assignment {
                        workspace,
                        output: area.output,
                        target: tile_target,
                    },
                );
                self.events.push_back(TilingEvent::Tiled {
                    window,
                    workspace,
                    output: area.output,
                    target: tile_target,
                    geometry,
                });
                Ok(Some(plan))
            }
        }
    }

    /// Removes one window from tiling and restores the floating geometry saved by
    /// the Window Manager Core before the first tile transition.
    pub fn untile(
        &mut self,
        wm: &mut WindowManager,
        window: WindowId,
    ) -> Result<UntilePlan, TilingError> {
        let placement = wm
            .window(window)
            .ok_or(TilingError::UnknownWindow(window))?
            .placement;
        if placement != WindowPlacement::Tiled {
            return Err(TilingError::NotTiled(window));
        }

        wm.set_placement(window, WindowPlacement::Floating)?;
        let geometry = wm
            .window(window)
            .ok_or(TilingError::UnknownWindow(window))?
            .geometry;
        let commands = [
            wm.request_move(window, geometry.x, geometry.y)?,
            wm.request_resize(window, geometry.width, geometry.height)?,
        ];

        self.assignments.remove(&window);
        self.events
            .push_back(TilingEvent::Released { window, geometry });
        Ok(UntilePlan {
            window,
            geometry,
            commands,
        })
    }

    /// Called when a manual move/resize starts. A tiled window immediately
    /// returns to floating; an already-floating window is left untouched.
    pub fn release_for_manual_operation(
        &mut self,
        wm: &mut WindowManager,
        window: WindowId,
    ) -> Result<Option<UntilePlan>, TilingError> {
        let placement = wm
            .window(window)
            .ok_or(TilingError::UnknownWindow(window))?
            .placement;
        if placement == WindowPlacement::Floating {
            self.assignments.remove(&window);
            return Ok(None);
        }
        self.untile(wm, window).map(Some)
    }

    /// Drops runtime assignment when a window is destroyed or moved to another
    /// workspace. No process/session persistence is introduced here.
    pub fn forget_window(&mut self, window: WindowId) {
        self.assignments.remove(&window);
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = TilingEvent> + '_ {
        self.events.drain(..)
    }

    /// Dispatches a plan through the existing backend-neutral WM sink contract.
    ///
    /// The initial X11 backend already implements this boundary; no X11 types or
    /// protocol details enter the tiling crate.
    pub fn dispatch_tile_plan(
        &self,
        wm: &WindowManager,
        sink: &mut impl BackendCommandSink,
        plan: &TilePlan,
    ) -> Result<(), TilingError> {
        for command in plan.commands() {
            wm.dispatch(sink, command)?;
        }
        Ok(())
    }

    pub fn dispatch_untile_plan(
        &self,
        wm: &WindowManager,
        sink: &mut impl BackendCommandSink,
        plan: &UntilePlan,
    ) -> Result<(), TilingError> {
        for command in plan.commands() {
            wm.dispatch(sink, command)?;
        }
        Ok(())
    }

    fn validate_membership(
        &self,
        workspaces: &WorkspaceManager,
        workspace: WorkspaceId,
        window: WindowId,
    ) -> Result<(), TilingError> {
        let actual = workspaces.workspace_of(window);
        if actual != Some(workspace) {
            return Err(TilingError::WorkspaceMismatch {
                window,
                requested: workspace,
                actual,
            });
        }
        Ok(())
    }

    fn window_constraints(
        &self,
        wm: &WindowManager,
        window: WindowId,
    ) -> Result<nexxus_wm::SizeConstraints, TilingError> {
        wm.window(window)
            .map(|window| window.constraints)
            .ok_or(TilingError::UnknownWindow(window))
    }

    fn apply_tiled_state(
        &self,
        wm: &mut WindowManager,
        window: WindowId,
    ) -> Result<(), TilingError> {
        let state = wm
            .window(window)
            .ok_or(TilingError::UnknownWindow(window))?
            .presentation;
        if state != PresentationState::Normal {
            return Err(TilingError::WindowManager(WmError::InvalidState {
                window,
                operation: "tile-fit",
                state,
            }));
        }
        wm.set_placement(window, WindowPlacement::Tiled)?;
        Ok(())
    }

    fn make_tile_plan(
        &self,
        wm: &WindowManager,
        window: WindowId,
        workspace: WorkspaceId,
        output: OutputId,
        target: TileTarget,
        geometry: Geometry,
    ) -> Result<TilePlan, TilingError> {
        let commands = [
            wm.request_move(window, geometry.x, geometry.y)?,
            wm.request_resize(window, geometry.width, geometry.height)?,
        ];
        Ok(TilePlan {
            window,
            workspace,
            output,
            target,
            geometry,
            commands,
        })
    }

    fn select_layout_slot(
        &self,
        workspace: WorkspaceId,
        output: OutputId,
        window: WindowId,
    ) -> Result<usize, TilingError> {
        let layout_len = self.layout_for(workspace).len();

        if let Some(Assignment {
            workspace: assigned_workspace,
            output: assigned_output,
            target: TileTarget::LayoutSlot(slot),
        }) = self.assignments.get(&window).copied()
        {
            if assigned_workspace == workspace && assigned_output == output && slot < layout_len {
                return Ok(slot);
            }
        }

        let used: BTreeSet<usize> = self
            .assignments
            .iter()
            .filter_map(|(assigned_window, assignment)| {
                if *assigned_window == window
                    || assignment.workspace != workspace
                    || assignment.output != output
                {
                    return None;
                }
                match assignment.target {
                    TileTarget::LayoutSlot(slot) if slot < layout_len => Some(slot),
                    TileTarget::LayoutSlot(_) | TileTarget::Snap(_) => None,
                }
            })
            .collect();

        (0..layout_len)
            .find(|slot| !used.contains(slot))
            .ok_or(TilingError::NoAvailableSlot { workspace, output })
    }
}
