//! Geometria determinística da barra no monitor primário.

use nexxus_ui::{LogicalRect, ScaleFactor};
use nexxus_workspaces::WorkspaceId;

/// Geometria lógica de um monitor. `primary` é a única propriedade usada para
/// decidir onde a Workspace Bar pode existir.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorGeometry {
    pub rect: LogicalRect,
    pub scale: ScaleFactor,
    pub primary: bool,
}

/// Métricas locais da barra. Valores são lógicos e portanto escalam pelo
/// `ScaleFactor` sem alterar ergonomia em HiDPI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkspaceBarMetrics {
    pub height: f32,
    pub top_margin: f32,
    pub side_margin: f32,
    pub padding: f32,
    pub gap: f32,
    pub min_button_width: f32,
    pub max_button_width: f32,
    pub settings_width: f32,
    pub icon_size: f32,
    pub border_width: f32,
}

impl Default for WorkspaceBarMetrics {
    fn default() -> Self {
        Self {
            height: 32.0,
            top_margin: 8.0,
            side_margin: 12.0,
            padding: 4.0,
            gap: 4.0,
            min_button_width: 48.0,
            max_button_width: 144.0,
            settings_width: 32.0,
            icon_size: 16.0,
            border_width: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceButtonLayout {
    pub id: WorkspaceId,
    pub rect: LogicalRect,
}

/// Layout local dos controles e posição global da janela suspensa.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceBarLayout {
    pub window: LogicalRect,
    pub workspaces: Vec<WorkspaceButtonLayout>,
    pub settings: LogicalRect,
}

impl WorkspaceBarLayout {
    /// Retorna `None` somente quando não existe monitor. Se nenhum monitor foi
    /// explicitamente marcado como primário, o primeiro é fallback técnico para
    /// não tornar a sessão inutilizável por uma topologia RandR incompleta.
    pub fn build(
        monitors: &[MonitorGeometry],
        workspace_labels: &[(WorkspaceId, &str)],
        metrics: WorkspaceBarMetrics,
    ) -> Option<Self> {
        let primary = monitors.iter().find(|monitor| monitor.primary).or_else(|| monitors.first())?;
        let max_width = (primary.rect.width - metrics.side_margin * 2.0).max(metrics.settings_width);
        let gap_count = workspace_labels.len();
        let fixed = metrics.padding * 2.0 + metrics.settings_width + metrics.gap * gap_count as f32;
        let available_for_workspaces = (max_width - fixed).max(0.0);

        let desired: Vec<f32> = workspace_labels
            .iter()
            .map(|(_, label)| {
                let text = label.chars().count() as f32 * 8.0 + metrics.padding * 4.0;
                text.clamp(metrics.min_button_width, metrics.max_button_width)
            })
            .collect();
        let desired_total: f32 = desired.iter().sum();
        let compression = if desired_total > available_for_workspaces && desired_total > 0.0 {
            available_for_workspaces / desired_total
        } else {
            1.0
        };

        let mut widths: Vec<f32> = desired
            .iter()
            .map(|width| (width * compression).max(28.0))
            .collect();
        let mut total = fixed + widths.iter().sum::<f32>();
        if total > max_width && !widths.is_empty() {
            let equal = (available_for_workspaces / widths.len() as f32).max(20.0);
            widths.fill(equal);
            total = fixed + widths.iter().sum::<f32>();
        }
        total = total.min(max_width).max(metrics.settings_width + metrics.padding * 2.0);

        let x = primary.rect.x + (primary.rect.width - total) * 0.5;
        let y = primary.rect.y + metrics.top_margin;
        let window = LogicalRect::new(x, y, total, metrics.height);
        let mut cursor = metrics.padding;
        let mut workspaces = Vec::with_capacity(workspace_labels.len());
        for ((id, _), width) in workspace_labels.iter().zip(widths) {
            let remaining = (total - cursor - metrics.settings_width - metrics.padding - metrics.gap).max(0.0);
            let width = width.min(remaining);
            let rect = LogicalRect::new(cursor, 0.0, width, metrics.height);
            workspaces.push(WorkspaceButtonLayout { id: *id, rect });
            cursor += width + metrics.gap;
        }
        let settings = LogicalRect::new(
            (total - metrics.padding - metrics.settings_width).max(0.0),
            0.0,
            metrics.settings_width,
            metrics.height,
        );
        Some(Self { window, workspaces, settings })
    }
}
