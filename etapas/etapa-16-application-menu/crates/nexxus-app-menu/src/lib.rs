//! Application Menu do Nexxus.
//!
//! O módulo mantém estado de navegação, favoritos, recentes, busca e integração
//! com o XDG Application Index. A apresentação gráfica é backend-neutral e o
//! plugin do painel usa exclusivamente o contrato público do Panel Core.

#![forbid(unsafe_code)]

use std::collections::{BTreeSet, VecDeque};
use std::fmt;

use nexxus_panel::{PanelPlugin, PluginApiVersion, PluginError, PluginMetadata};
use nexxus_xdg_application_index::{
    ApplicationRecord, IconReference, IndexSnapshot, LaunchCommand, LaunchContext, MainCategory,
};

pub const APPLICATION_MENU_PLUGIN_ID: &str = "nexxus.application-menu";
pub const APPLICATION_MENU_DISPLAY_NAME: &str = "Application Menu";
pub const DEFAULT_RECENT_LIMIT: usize = 12;

/// Seção lógica selecionada pelo usuário no menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSection {
    Favorites,
    Recent,
    All,
    Category(MainCategory),
}

/// Modo de apresentação do catálogo. O renderer concreto interpreta este estado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    List,
    Grid,
}

/// Tamanhos de ícone configuráveis sem acoplar o menu ao backend gráfico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    Small,
    Medium,
    Large,
}

impl IconSize {
    pub fn logical_px(self) -> u32 {
        match self {
            Self::Small => 20,
            Self::Medium => 28,
            Self::Large => 40,
        }
    }
}

/// Entrada pronta para ser consumida pela UI própria do Nexxus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    pub desktop_id: String,
    pub name: String,
    pub icon: IconReference,
    pub favorite: bool,
    pub recent: bool,
}

/// Estado observável do menu. Favoritos usam conjunto ordenado para persistência
/// determinística; recentes preservam ordem MRU e não admitem duplicatas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationMenuState {
    open: bool,
    query: String,
    section: MenuSection,
    view_mode: ViewMode,
    icon_size: IconSize,
    favorites: BTreeSet<String>,
    recent: VecDeque<String>,
    recent_limit: usize,
}

impl Default for ApplicationMenuState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            section: MenuSection::Favorites,
            view_mode: ViewMode::List,
            icon_size: IconSize::Medium,
            favorites: BTreeSet::new(),
            recent: VecDeque::new(),
            recent_limit: DEFAULT_RECENT_LIMIT,
        }
    }
}

impl ApplicationMenuState {
    /// Abre o menu e preserva a seção atual; o foco do campo de busca pertence à UI.
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }

    pub fn section(&self) -> MenuSection {
        self.section
    }

    pub fn set_section(&mut self, section: MenuSection) {
        self.section = section;
        self.query.clear();
    }

    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn icon_size(&self) -> IconSize {
        self.icon_size
    }

    pub fn set_icon_size(&mut self, size: IconSize) {
        self.icon_size = size;
    }

    /// Alterna favorito sem criar referência duplicada.
    pub fn toggle_favorite(&mut self, desktop_id: &str) -> bool {
        if self.favorites.remove(desktop_id) {
            false
        } else {
            self.favorites.insert(desktop_id.to_owned());
            true
        }
    }

    pub fn is_favorite(&self, desktop_id: &str) -> bool {
        self.favorites.contains(desktop_id)
    }

    pub fn favorites(&self) -> impl Iterator<Item = &str> {
        self.favorites.iter().map(String::as_str)
    }

    /// Registra uso em ordem MRU e aplica o limite configurado.
    pub fn mark_recent(&mut self, desktop_id: &str) {
        self.recent.retain(|value| value != desktop_id);
        self.recent.push_front(desktop_id.to_owned());
        while self.recent.len() > self.recent_limit {
            self.recent.pop_back();
        }
    }

    pub fn recent(&self) -> impl Iterator<Item = &str> {
        self.recent.iter().map(String::as_str)
    }

    pub fn set_recent_limit(&mut self, limit: usize) {
        self.recent_limit = limit.max(1);
        while self.recent.len() > self.recent_limit {
            self.recent.pop_back();
        }
    }

    /// Produz a lista visível a partir do snapshot autoritativo da Etapa 12.
    /// Busca sempre tem precedência sobre a seção selecionada.
    pub fn visible_entries(&self, snapshot: &IndexSnapshot) -> Vec<MenuEntry> {
        let records: Vec<&ApplicationRecord> = if !self.query.trim().is_empty() {
            snapshot.search(&self.query)
        } else {
            match self.section {
                MenuSection::Favorites => self
                    .favorites
                    .iter()
                    .filter_map(|id| snapshot.by_id(id))
                    .filter(|record| record.is_visible())
                    .collect(),
                MenuSection::Recent => self
                    .recent
                    .iter()
                    .filter_map(|id| snapshot.by_id(id))
                    .filter(|record| record.is_visible())
                    .collect(),
                MenuSection::All => snapshot.visible_entries().collect(),
                MenuSection::Category(category) => snapshot.category(category).collect(),
            }
        };

        records
            .into_iter()
            .map(|record| MenuEntry {
                desktop_id: record.id.as_str().to_owned(),
                name: record.name.clone(),
                icon: record.icon.clone(),
                favorite: self.is_favorite(record.id.as_str()),
                recent: self.recent.iter().any(|id| id == record.id.as_str()),
            })
            .collect()
    }

    /// Expande `Exec` sem shell e só registra recente quando há comando válido.
    pub fn launch_command(
        &mut self,
        snapshot: &IndexSnapshot,
        desktop_id: &str,
        context: &LaunchContext,
    ) -> Result<LaunchCommand, ApplicationMenuError> {
        let record = snapshot
            .by_id(desktop_id)
            .filter(|record| record.is_visible())
            .ok_or_else(|| ApplicationMenuError::UnknownApplication(desktop_id.to_owned()))?;
        let template = record
            .exec
            .as_ref()
            .ok_or_else(|| ApplicationMenuError::NotDirectlyLaunchable(desktop_id.to_owned()))?;
        let command = template.expand(
            context,
            &record.name,
            record.icon.exec_icon_value(),
            &record.desktop_file,
        );
        self.mark_recent(desktop_id);
        Ok(command)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationMenuError {
    UnknownApplication(String),
    NotDirectlyLaunchable(String),
}

impl fmt::Display for ApplicationMenuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownApplication(id) => write!(formatter, "unknown application '{id}'"),
            Self::NotDirectlyLaunchable(id) => {
                write!(formatter, "application '{id}' has no direct Exec command")
            }
        }
    }
}

impl std::error::Error for ApplicationMenuError {}

/// Adaptador do menu para o lifecycle do Panel Core.
///
/// Renderização e eventos de clique/teclado permanecem no host/UI; este objeto
/// apenas estabelece identidade, compatibilidade de API e estado de instância.
pub struct ApplicationMenuPanelPlugin {
    loaded_instance: Option<String>,
}

impl Default for ApplicationMenuPanelPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationMenuPanelPlugin {
    pub fn new() -> Self {
        Self {
            loaded_instance: None,
        }
    }

    pub fn loaded_instance(&self) -> Option<&str> {
        self.loaded_instance.as_deref()
    }
}

impl PanelPlugin for ApplicationMenuPanelPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            plugin_id: APPLICATION_MENU_PLUGIN_ID.to_owned(),
            display_name: APPLICATION_MENU_DISPLAY_NAME.to_owned(),
            api: PluginApiVersion::CURRENT,
        }
    }

    fn load(&mut self, instance_id: &str) -> Result<(), PluginError> {
        if self.loaded_instance.is_some() {
            return Err(PluginError::Startup(
                "application menu instance is already loaded".to_owned(),
            ));
        }
        self.loaded_instance = Some(instance_id.to_owned());
        Ok(())
    }

    fn unload(&mut self, instance_id: &str) -> Result<(), PluginError> {
        match self.loaded_instance.as_deref() {
            Some(current) if current == instance_id => {
                self.loaded_instance = None;
                Ok(())
            }
            _ => Err(PluginError::Shutdown(
                "application menu instance does not match loaded instance".to_owned(),
            )),
        }
    }
}
