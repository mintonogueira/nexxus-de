//! XDG main-category normalization and mapping to Nexxus visual fallbacks.

use nexxus_assets::AppCategory;

/// Main categories recognized by the current Freedesktop menu category registry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MainCategory {
    AudioVideo,
    Audio,
    Video,
    Development,
    Education,
    HealthFitness,
    Game,
    Graphics,
    Network,
    Office,
    Science,
    Settings,
    System,
    Utility,
    Other,
}

impl MainCategory {
    /// Converts an exact, case-sensitive XDG category token into a main category.
    pub fn from_xdg(value: &str) -> Option<Self> {
        Some(match value {
            "AudioVideo" => Self::AudioVideo,
            "Audio" => Self::Audio,
            "Video" => Self::Video,
            "Development" => Self::Development,
            "Education" => Self::Education,
            "HealthFitness" => Self::HealthFitness,
            "Game" => Self::Game,
            "Graphics" => Self::Graphics,
            "Network" => Self::Network,
            "Office" => Self::Office,
            "Science" => Self::Science,
            "Settings" => Self::Settings,
            "System" => Self::System,
            "Utility" => Self::Utility,
            _ => return None,
        })
    }

    /// Maps the XDG category space to the coarser fallback catalog from Stage 08.
    pub(crate) fn asset_category(self) -> AppCategory {
        match self {
            Self::AudioVideo | Self::Audio | Self::Video => AppCategory::AudioVideo,
            Self::Development => AppCategory::Development,
            Self::Education | Self::HealthFitness | Self::Science => AppCategory::Education,
            Self::Game => AppCategory::Game,
            Self::Graphics => AppCategory::Graphics,
            Self::Network => AppCategory::Network,
            Self::Office => AppCategory::Office,
            Self::Settings => AppCategory::Settings,
            Self::System => AppCategory::System,
            Self::Utility => AppCategory::Utility,
            Self::Other => AppCategory::Other,
        }
    }
}
