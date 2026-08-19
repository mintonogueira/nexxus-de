//! Core do painel inferior fixo do Nexxus.
//!
//! Este crate concentra política do painel, layout, lifecycle de plugins,
//! persistência e contrato X11/EWMH. O transporte X11/Wayland permanece nos
//! backends gráficos, preservando a modularidade do projeto.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const PANEL_HEIGHT_MIN: u32 = 24;
pub const PANEL_HEIGHT_DEFAULT: u32 = 40;
pub const PANEL_HEIGHT_MAX: u32 = 96;
pub const PANEL_SCHEMA_VERSION: u32 = 1;
pub const PANEL_PLUGIN_API_MAJOR: u16 = 1;
pub const PANEL_PLUGIN_API_MINOR: u16 = 0;

/// Fator de escala físico aplicado à altura lógica do painel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleFactor(f32);

impl ScaleFactor {
    pub fn new(value: f32) -> Self {
        Self(if value.is_finite() && value > 0.0 { value } else { 1.0 })
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

/// Retângulo em coordenadas do backend gráfico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Métricas proporcionais derivadas da altura final do painel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelMetrics {
    pub icon_size: u32,
    pub padding: u32,
    pub hit_target: u32,
    pub spacing: u32,
}

impl PanelMetrics {
    /// Mantém ícones, padding e áreas clicáveis proporcionais à altura configurada.
    pub fn from_height(height: u32) -> Self {
        let icon_size = ((height as f32) * 0.55).round() as u32;
        let padding = ((height as f32) * 0.12).round() as u32;
        let hit_target = height.saturating_sub(padding.saturating_mul(2)).max(icon_size);
        let spacing = ((height as f32) * 0.08).round() as u32;
        Self {
            icon_size: icon_size.max(12),
            padding: padding.max(2),
            hit_target,
            spacing: spacing.max(2),
        }
    }
}

/// Geometria final do painel no output informado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelGeometry {
    pub rect: Rect,
    pub metrics: PanelMetrics,
}

impl PanelGeometry {
    /// Posiciona o painel exatamente na borda inferior do output.
    pub fn bottom(output: Rect, config: &PanelConfig, scale: ScaleFactor) -> Result<Self, PanelError> {
        config.validate()?;
        let physical_height = ((config.height as f32) * scale.get()).round().max(1.0) as u32;
        let y = output
            .y
            .saturating_add(output.height.saturating_sub(physical_height) as i32);
        Ok(Self {
            rect: Rect {
                x: output.x,
                y,
                width: output.width,
                height: physical_height,
            },
            metrics: PanelMetrics::from_height(physical_height),
        })
    }
}

/// Zona lógica do painel. As zonas organizam, mas não bloqueiam movimentação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PanelZone {
    Start,
    Center,
    End,
}

impl PanelZone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }

    pub fn parse(value: &str) -> Result<Self, PanelError> {
        match value {
            "start" => Ok(Self::Start),
            "center" => Ok(Self::Center),
            "end" => Ok(Self::End),
            other => Err(PanelError::InvalidZone(other.to_owned())),
        }
    }
}

/// Placement persistido de uma instância de plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelPluginConfig {
    pub instance_id: String,
    pub plugin_id: String,
    pub zone: PanelZone,
    pub order: u32,
    pub enabled: bool,
}

impl PanelPluginConfig {
    pub fn new(
        instance_id: impl Into<String>,
        plugin_id: impl Into<String>,
        zone: PanelZone,
        order: u32,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            plugin_id: plugin_id.into(),
            zone,
            order,
            enabled: true,
        }
    }
}

/// Configuração versionada do painel independente do backend gráfico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelConfig {
    pub schema_version: u32,
    pub height: u32,
    pub plugins: Vec<PanelPluginConfig>,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            schema_version: PANEL_SCHEMA_VERSION,
            height: PANEL_HEIGHT_DEFAULT,
            plugins: Vec::new(),
        }
    }
}

impl PanelConfig {
    /// Rejeita estado ambíguo ou fora dos limites antes de renderização/persistência.
    pub fn validate(&self) -> Result<(), PanelError> {
        if self.schema_version != PANEL_SCHEMA_VERSION {
            return Err(PanelError::UnsupportedSchema(self.schema_version));
        }
        if !(PANEL_HEIGHT_MIN..=PANEL_HEIGHT_MAX).contains(&self.height) {
            return Err(PanelError::InvalidHeight(self.height));
        }

        let mut ids = HashSet::new();
        for plugin in &self.plugins {
            if plugin.instance_id.trim().is_empty() || plugin.plugin_id.trim().is_empty() {
                return Err(PanelError::EmptyPluginIdentifier);
            }
            if !ids.insert(plugin.instance_id.as_str()) {
                return Err(PanelError::DuplicateInstance(plugin.instance_id.clone()));
            }
        }
        Ok(())
    }

    pub fn set_height(&mut self, height: u32) -> Result<(), PanelError> {
        if !(PANEL_HEIGHT_MIN..=PANEL_HEIGHT_MAX).contains(&height) {
            return Err(PanelError::InvalidHeight(height));
        }
        self.height = height;
        Ok(())
    }

    /// Adiciona uma instância e normaliza a ordem da zona correspondente.
    pub fn add_plugin(&mut self, plugin: PanelPluginConfig) -> Result<(), PanelError> {
        if self
            .plugins
            .iter()
            .any(|item| item.instance_id == plugin.instance_id)
        {
            return Err(PanelError::DuplicateInstance(plugin.instance_id));
        }
        self.plugins.push(plugin);
        self.normalize_orders();
        Ok(())
    }

    pub fn remove_plugin(&mut self, instance_id: &str) -> Result<PanelPluginConfig, PanelError> {
        let index = self
            .plugins
            .iter()
            .position(|item| item.instance_id == instance_id)
            .ok_or_else(|| PanelError::UnknownInstance(instance_id.to_owned()))?;
        let removed = self.plugins.remove(index);
        self.normalize_orders();
        Ok(removed)
    }

    /// Move livremente uma instância entre zonas e posições.
    pub fn move_plugin(
        &mut self,
        instance_id: &str,
        zone: PanelZone,
        target_index: usize,
    ) -> Result<(), PanelError> {
        let index = self
            .plugins
            .iter()
            .position(|item| item.instance_id == instance_id)
            .ok_or_else(|| PanelError::UnknownInstance(instance_id.to_owned()))?;
        let mut plugin = self.plugins.remove(index);
        plugin.zone = zone;

        let mut zone_indices: Vec<usize> = self
            .plugins
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| (item.zone == zone).then_some(idx))
            .collect();
        zone_indices.sort_unstable();

        let insertion = if target_index >= zone_indices.len() {
            zone_indices
                .last()
                .map_or(self.plugins.len(), |idx| idx + 1)
        } else {
            zone_indices[target_index]
        };

        self.plugins.insert(insertion, plugin);
        self.normalize_orders();
        Ok(())
    }

    /// Retorna somente plugins habilitados em ordem visual determinística.
    pub fn ordered_plugins(&self) -> Vec<&PanelPluginConfig> {
        let mut plugins: Vec<_> = self.plugins.iter().filter(|item| item.enabled).collect();
        plugins.sort_by_key(|item| (item.zone, item.order, item.instance_id.as_str()));
        plugins
    }

    fn normalize_orders(&mut self) {
        for zone in [PanelZone::Start, PanelZone::Center, PanelZone::End] {
            let mut indices: Vec<_> = self
                .plugins
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| (item.zone == zone).then_some(idx))
                .collect();
            indices.sort_by_key(|idx| {
                (
                    self.plugins[*idx].order,
                    self.plugins[*idx].instance_id.clone(),
                )
            });
            for (order, idx) in indices.into_iter().enumerate() {
                self.plugins[idx].order = order as u32;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelError {
    InvalidHeight(u32),
    UnsupportedSchema(u32),
    DuplicateInstance(String),
    UnknownInstance(String),
    EmptyPluginIdentifier,
    InvalidZone(String),
}

impl fmt::Display for PanelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeight(value) => write!(
                f,
                "panel height {value} is outside {PANEL_HEIGHT_MIN}..={PANEL_HEIGHT_MAX}"
            ),
            Self::UnsupportedSchema(value) => {
                write!(f, "unsupported panel schema version {value}")
            }
            Self::DuplicateInstance(value) => {
                write!(f, "duplicate plugin instance '{value}'")
            }
            Self::UnknownInstance(value) => write!(f, "unknown plugin instance '{value}'"),
            Self::EmptyPluginIdentifier => write!(f, "plugin identifiers must not be empty"),
            Self::InvalidZone(value) => write!(f, "invalid panel zone '{value}'"),
        }
    }
}

impl std::error::Error for PanelError {}

/// Versão do contrato lógico de plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginApiVersion {
    pub major: u16,
    pub minor: u16,
}

impl PluginApiVersion {
    pub const CURRENT: Self = Self {
        major: PANEL_PLUGIN_API_MAJOR,
        minor: PANEL_PLUGIN_API_MINOR,
    };

    pub fn compatible_with(self, host: Self) -> bool {
        self.major == host.major && self.minor <= host.minor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMetadata {
    pub plugin_id: String,
    pub display_name: String,
    pub api: PluginApiVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginEvent {
    Loaded { instance_id: String },
    Unloaded { instance_id: String },
}

/// Contrato in-process para plugins pequenos do painel.
///
/// Integrações de maior risco podem futuramente ser isoladas via IPC sem mudar
/// o modelo de placement/lifecycle exposto pelo host.
pub trait PanelPlugin: Send {
    fn metadata(&self) -> PluginMetadata;
    fn load(&mut self, instance_id: &str) -> Result<(), PluginError>;
    fn unload(&mut self, instance_id: &str) -> Result<(), PluginError>;
}

struct LoadedPlugin {
    plugin_id: String,
    plugin: Box<dyn PanelPlugin>,
}

/// Registry leve de lifecycle. Falha de unload preserva a instância no host.
pub struct PluginRegistry {
    loaded: BTreeMap<String, LoadedPlugin>,
    events: Vec<PluginEvent>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            loaded: BTreeMap::new(),
            events: Vec::new(),
        }
    }

    pub fn load(
        &mut self,
        instance_id: impl Into<String>,
        mut plugin: Box<dyn PanelPlugin>,
    ) -> Result<(), PluginError> {
        let instance_id = instance_id.into();
        if instance_id.trim().is_empty() {
            return Err(PluginError::InvalidInstanceId);
        }
        if self.loaded.contains_key(&instance_id) {
            return Err(PluginError::AlreadyLoaded(instance_id));
        }

        let metadata = plugin.metadata();
        if metadata.plugin_id.trim().is_empty() {
            return Err(PluginError::InvalidPluginId);
        }
        if !metadata.api.compatible_with(PluginApiVersion::CURRENT) {
            return Err(PluginError::IncompatibleApi {
                plugin: metadata.api,
                host: PluginApiVersion::CURRENT,
            });
        }

        plugin.load(&instance_id)?;
        self.loaded.insert(
            instance_id.clone(),
            LoadedPlugin {
                plugin_id: metadata.plugin_id,
                plugin,
            },
        );
        self.events.push(PluginEvent::Loaded { instance_id });
        Ok(())
    }

    pub fn unload(&mut self, instance_id: &str) -> Result<(), PluginError> {
        let mut loaded = self
            .loaded
            .remove(instance_id)
            .ok_or_else(|| PluginError::NotLoaded(instance_id.to_owned()))?;

        if let Err(error) = loaded.plugin.unload(instance_id) {
            self.loaded.insert(instance_id.to_owned(), loaded);
            return Err(error);
        }

        self.events.push(PluginEvent::Unloaded {
            instance_id: instance_id.to_owned(),
        });
        Ok(())
    }

    pub fn is_loaded(&self, instance_id: &str) -> bool {
        self.loaded.contains_key(instance_id)
    }

    pub fn plugin_id(&self, instance_id: &str) -> Option<&str> {
        self.loaded
            .get(instance_id)
            .map(|item| item.plugin_id.as_str())
    }

    pub fn drain_events(&mut self) -> Vec<PluginEvent> {
        std::mem::take(&mut self.events)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    InvalidInstanceId,
    InvalidPluginId,
    AlreadyLoaded(String),
    NotLoaded(String),
    IncompatibleApi {
        plugin: PluginApiVersion,
        host: PluginApiVersion,
    },
    Startup(String),
    Shutdown(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInstanceId => write!(f, "plugin instance id must not be empty"),
            Self::InvalidPluginId => write!(f, "plugin id must not be empty"),
            Self::AlreadyLoaded(id) => write!(f, "plugin instance '{id}' is already loaded"),
            Self::NotLoaded(id) => write!(f, "plugin instance '{id}' is not loaded"),
            Self::IncompatibleApi { plugin, host } => write!(
                f,
                "plugin API {}.{} is incompatible with host {}.{}",
                plugin.major, plugin.minor, host.major, host.minor
            ),
            Self::Startup(message) => write!(f, "plugin startup failed: {message}"),
            Self::Shutdown(message) => write!(f, "plugin shutdown failed: {message}"),
        }
    }
}

impl std::error::Error for PluginError {}

/// Persistência atômica de configuração do painel.
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Grava em arquivo temporário irmão, sincroniza e renomeia atomicamente.
    pub fn save(&self, config: &PanelConfig) -> Result<(), PersistenceError> {
        config.validate()?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp = self.path.with_extension("tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)?;
        file.write_all(encode_config(config).as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp, &self.path)?;

        if let Some(parent) = self.path.parent() {
            if let Ok(dir) = File::open(parent) {
                let _ = dir.sync_all();
            }
        }
        Ok(())
    }

    pub fn load(&self) -> Result<PanelConfig, PersistenceError> {
        let mut data = String::new();
        File::open(&self.path)?.read_to_string(&mut data)?;
        let config = decode_config(&data)?;
        config.validate()?;
        Ok(config)
    }
}

fn encode_config(config: &PanelConfig) -> String {
    let mut output = format!(
        "schema={}\nheight={}\n",
        config.schema_version, config.height
    );
    for plugin in &config.plugins {
        output.push_str("plugin=");
        output.push_str(&escape_field(&plugin.instance_id));
        output.push('|');
        output.push_str(&escape_field(&plugin.plugin_id));
        output.push('|');
        output.push_str(plugin.zone.as_str());
        output.push('|');
        output.push_str(&plugin.order.to_string());
        output.push('|');
        output.push_str(if plugin.enabled { "1" } else { "0" });
        output.push('\n');
    }
    output
}

fn decode_config(data: &str) -> Result<PanelConfig, PersistenceError> {
    let mut schema = None;
    let mut height = None;
    let mut plugins = Vec::new();

    for (line_no, raw) in data.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(value) = line.strip_prefix("schema=") {
            schema = Some(
                value
                    .parse()
                    .map_err(|_| PersistenceError::Malformed(line_no + 1))?,
            );
        } else if let Some(value) = line.strip_prefix("height=") {
            height = Some(
                value
                    .parse()
                    .map_err(|_| PersistenceError::Malformed(line_no + 1))?,
            );
        } else if let Some(value) = line.strip_prefix("plugin=") {
            let fields = split_escaped(value)?;
            if fields.len() != 5 {
                return Err(PersistenceError::Malformed(line_no + 1));
            }
            plugins.push(PanelPluginConfig {
                instance_id: fields[0].clone(),
                plugin_id: fields[1].clone(),
                zone: PanelZone::parse(&fields[2])?,
                order: fields[3]
                    .parse()
                    .map_err(|_| PersistenceError::Malformed(line_no + 1))?,
                enabled: match fields[4].as_str() {
                    "1" => true,
                    "0" => false,
                    _ => return Err(PersistenceError::Malformed(line_no + 1)),
                },
            });
        } else {
            return Err(PersistenceError::Malformed(line_no + 1));
        }
    }

    Ok(PanelConfig {
        schema_version: schema.ok_or(PersistenceError::MissingField("schema"))?,
        height: height.ok_or(PersistenceError::MissingField("height"))?,
        plugins,
    })
}

fn escape_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "\\n")
}

fn split_escaped(value: &str) -> Result<Vec<String>, PersistenceError> {
    let mut fields = vec![String::new()];
    let mut escaped = false;

    for ch in value.chars() {
        if escaped {
            match ch {
                'n' => fields.last_mut().expect("field exists").push('\n'),
                '\\' | '|' => fields.last_mut().expect("field exists").push(ch),
                _ => return Err(PersistenceError::InvalidEscape),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '|' {
            fields.push(String::new());
        } else {
            fields.last_mut().expect("field exists").push(ch);
        }
    }

    if escaped {
        return Err(PersistenceError::InvalidEscape);
    }
    Ok(fields)
}

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Panel(PanelError),
    Malformed(usize),
    MissingField(&'static str),
    InvalidEscape,
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Panel(error) => write!(f, "invalid panel configuration: {error}"),
            Self::Malformed(line) => {
                write!(f, "malformed panel configuration at line {line}")
            }
            Self::MissingField(field) => {
                write!(f, "missing panel configuration field '{field}'")
            }
            Self::InvalidEscape => {
                write!(f, "invalid escape sequence in panel configuration")
            }
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<PanelError> for PersistenceError {
    fn from(value: PanelError) -> Self {
        Self::Panel(value)
    }
}

/// Hints que o presenter X11 deve publicar na janela do painel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X11DockHints {
    /// `_NET_WM_STRUT`: left, right, top, bottom.
    pub strut: [u32; 4],
    /// `_NET_WM_STRUT_PARTIAL` na ordem definida pelo EWMH.
    pub strut_partial: [u32; 12],
    /// Deve corresponder a `_NET_WM_WINDOW_TYPE_DOCK` antes do map.
    pub window_type_dock: bool,
}

impl X11DockHints {
    /// Calcula a reserva inferior usando coordenadas da root window.
    pub fn for_bottom_panel(root: Rect, panel: Rect) -> Self {
        let root_right = root.x.saturating_add(root.width as i32);
        let root_bottom = root.y.saturating_add(root.height as i32);
        let panel_top = panel.y.clamp(root.y, root_bottom);
        let panel_left = panel.x.clamp(root.x, root_right);
        let panel_right = panel
            .x
            .saturating_add(panel.width as i32)
            .clamp(root.x, root_right);

        let bottom = root_bottom.saturating_sub(panel_top) as u32;
        let start_x = panel_left.saturating_sub(root.x) as u32;
        let end_x = panel_right
            .saturating_sub(root.x)
            .saturating_sub(1)
            .max(0) as u32;

        Self {
            strut: [0, 0, 0, bottom],
            strut_partial: [0, 0, 0, bottom, 0, 0, 0, 0, 0, 0, start_x, end_x],
            window_type_dock: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct DummyPlugin {
        fail_unload: bool,
    }

    impl PanelPlugin for DummyPlugin {
        fn metadata(&self) -> PluginMetadata {
            PluginMetadata {
                plugin_id: "dummy".into(),
                display_name: "Dummy".into(),
                api: PluginApiVersion::CURRENT,
            }
        }

        fn load(&mut self, _: &str) -> Result<(), PluginError> {
            Ok(())
        }

        fn unload(&mut self, _: &str) -> Result<(), PluginError> {
            if self.fail_unload {
                Err(PluginError::Shutdown("busy".into()))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn panel_is_always_at_bottom() {
        let output = Rect {
            x: 10,
            y: 20,
            width: 1920,
            height: 1080,
        };
        let geometry = PanelGeometry::bottom(
            output,
            &PanelConfig::default(),
            ScaleFactor::new(1.0),
        )
        .unwrap();
        assert_eq!(
            geometry.rect.y + geometry.rect.height as i32,
            output.y + output.height as i32
        );
    }

    #[test]
    fn scaling_changes_height_and_metrics() {
        let output = Rect {
            x: 0,
            y: 0,
            width: 1000,
            height: 800,
        };
        let config = PanelConfig::default();
        let one = PanelGeometry::bottom(output, &config, ScaleFactor::new(1.0)).unwrap();
        let two = PanelGeometry::bottom(output, &config, ScaleFactor::new(2.0)).unwrap();
        assert_eq!(two.rect.height, one.rect.height * 2);
        assert!(two.metrics.icon_size > one.metrics.icon_size);
        assert!(two.metrics.hit_target > one.metrics.hit_target);
    }

    #[test]
    fn plugins_can_move_between_zones() {
        let mut config = PanelConfig::default();
        config
            .add_plugin(PanelPluginConfig::new("a", "menu", PanelZone::Start, 0))
            .unwrap();
        config
            .add_plugin(PanelPluginConfig::new("b", "clock", PanelZone::End, 0))
            .unwrap();
        config.move_plugin("b", PanelZone::Start, 0).unwrap();
        let ordered = config.ordered_plugins();
        assert_eq!(ordered[0].instance_id, "b");
        assert_eq!(ordered[1].instance_id, "a");
    }

    #[test]
    fn duplicate_instance_is_rejected() {
        let mut config = PanelConfig::default();
        config
            .add_plugin(PanelPluginConfig::new(
                "same",
                "one",
                PanelZone::Start,
                0,
            ))
            .unwrap();
        assert!(matches!(
            config.add_plugin(PanelPluginConfig::new(
                "same",
                "two",
                PanelZone::End,
                0
            )),
            Err(PanelError::DuplicateInstance(_))
        ));
    }

    #[test]
    fn load_and_unload_emit_events() {
        let mut registry = PluginRegistry::new();
        registry
            .load(
                "dummy-1",
                Box::new(DummyPlugin { fail_unload: false }),
            )
            .unwrap();
        assert!(registry.is_loaded("dummy-1"));
        registry.unload("dummy-1").unwrap();
        assert!(!registry.is_loaded("dummy-1"));
        assert_eq!(registry.drain_events().len(), 2);
    }

    #[test]
    fn unload_failure_rolls_back_registry_state() {
        let mut registry = PluginRegistry::new();
        registry
            .load("dummy-1", Box::new(DummyPlugin { fail_unload: true }))
            .unwrap();
        assert!(registry.unload("dummy-1").is_err());
        assert!(registry.is_loaded("dummy-1"));
    }

    #[test]
    fn persistence_round_trip_preserves_layout() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("nexxus-panel-{suffix}.conf"));
        let store = ConfigStore::new(&path);
        let mut config = PanelConfig::default();
        config.set_height(52).unwrap();
        config
            .add_plugin(PanelPluginConfig::new(
                "clock|main",
                "clock",
                PanelZone::End,
                0,
            ))
            .unwrap();
        store.save(&config).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded, config);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn x11_reserves_only_bottom_panel_area() {
        let root = Rect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let panel = Rect {
            x: 0,
            y: 1040,
            width: 1920,
            height: 40,
        };
        let hints = X11DockHints::for_bottom_panel(root, panel);
        assert_eq!(hints.strut, [0, 0, 0, 40]);
        assert_eq!(hints.strut_partial[10], 0);
        assert_eq!(hints.strut_partial[11], 1919);
        assert!(hints.window_type_dock);
    }
}
