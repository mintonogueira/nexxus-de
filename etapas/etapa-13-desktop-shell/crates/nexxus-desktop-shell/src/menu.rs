//! Desktop context-menu model built from the shared XDG application index.
//!
//! The menu owns no application database. Every application/category view is
//! derived from the immutable Stage 12 snapshot so Panel/Menu/Finder can share
//! one source of truth.

use nexxus_xdg_application_index::{IndexSnapshot, MainCategory};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MenuPage {
    Root,
    Applications,
    Category(MainCategory),
    CreateLauncher,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MenuAction {
    OpenApplications,
    OpenCategory(MainCategory),
    LaunchApplication(String),
    OpenTerminal,
    OpenFileManager,
    CreateFolder,
    OpenCreateLauncher,
    PinLauncher(String),
    OpenDesktopSettings,
    Back,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuEntry {
    pub label: String,
    pub action: MenuAction,
}

pub fn entries(page: &MenuPage, snapshot: &IndexSnapshot) -> Vec<MenuEntry> {
    match page {
        MenuPage::Root => vec![
            entry("Applications", MenuAction::OpenApplications),
            entry("Terminal", MenuAction::OpenTerminal),
            entry("File Manager", MenuAction::OpenFileManager),
            entry("Create Folder", MenuAction::CreateFolder),
            entry("Create Launcher", MenuAction::OpenCreateLauncher),
            entry("Desktop Settings", MenuAction::OpenDesktopSettings),
        ],
        MenuPage::Applications => {
            let mut result = Vec::new();
            for category in CATEGORY_ORDER {
                if snapshot.category(*category).next().is_some() {
                    result.push(entry(category_label(*category), MenuAction::OpenCategory(*category)));
                }
            }
            result.push(entry("Back", MenuAction::Back));
            result
        }
        MenuPage::Category(category) => {
            let mut result = snapshot
                .category(*category)
                .map(|record| {
                    entry(
                        &record.name,
                        MenuAction::LaunchApplication(record.id.as_str().to_owned()),
                    )
                })
                .collect::<Vec<_>>();
            result.push(entry("Back", MenuAction::Back));
            result
        }
        MenuPage::CreateLauncher => {
            let mut result = snapshot
                .visible_entries()
                .map(|record| {
                    entry(
                        &record.name,
                        MenuAction::PinLauncher(record.id.as_str().to_owned()),
                    )
                })
                .collect::<Vec<_>>();
            result.sort_by_key(|item| item.label.to_lowercase());
            result.push(entry("Back", MenuAction::Back));
            result
        }
    }
}

fn entry(label: &str, action: MenuAction) -> MenuEntry {
    MenuEntry {
        label: label.to_owned(),
        action,
    }
}

pub fn category_label(category: MainCategory) -> &'static str {
    match category {
        MainCategory::AudioVideo => "Audio & Video",
        MainCategory::Audio => "Audio",
        MainCategory::Video => "Video",
        MainCategory::Development => "Development",
        MainCategory::Education => "Education",
        MainCategory::HealthFitness => "Health & Fitness",
        MainCategory::Game => "Games",
        MainCategory::Graphics => "Graphics",
        MainCategory::Network => "Internet & Network",
        MainCategory::Office => "Office",
        MainCategory::Science => "Science",
        MainCategory::Settings => "Settings",
        MainCategory::System => "System",
        MainCategory::Utility => "Utilities",
        MainCategory::Other => "Other",
    }
}

const CATEGORY_ORDER: &[MainCategory] = &[
    MainCategory::Development,
    MainCategory::Graphics,
    MainCategory::Network,
    MainCategory::Office,
    MainCategory::AudioVideo,
    MainCategory::Audio,
    MainCategory::Video,
    MainCategory::Education,
    MainCategory::Science,
    MainCategory::HealthFitness,
    MainCategory::Game,
    MainCategory::Utility,
    MainCategory::System,
    MainCategory::Settings,
    MainCategory::Other,
];
