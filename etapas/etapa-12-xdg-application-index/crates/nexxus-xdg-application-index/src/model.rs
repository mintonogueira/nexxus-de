//! Immutable application records, category views, diagnostics and index deltas.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;

use crate::{ApplicationSource, ExecTemplate, IconReference, MainCategory};

/// Canonical Desktop File ID including the `.desktop` suffix.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DesktopId(String);

impl DesktopId {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DesktopId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Normalized metadata from one effective XDG application entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRecord {
    pub id: DesktopId,
    pub desktop_file: PathBuf,
    pub source: ApplicationSource,
    pub name: String,
    pub exec: Option<ExecTemplate>,
    pub dbus_activatable: bool,
    pub icon: IconReference,
    pub categories: Vec<String>,
    pub main_categories: Vec<MainCategory>,
    pub keywords: Vec<String>,
    pub no_display: bool,
    pub visible_in_current_desktop: bool,
}

impl ApplicationRecord {
    /// `NoDisplay` and Only/NotShowIn affect presentation but do not delete the
    /// effective entry from the underlying catalog.
    pub fn is_visible(&self) -> bool {
        !self.no_display && self.visible_in_current_desktop
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexDiagnosticKind {
    Io,
    TooLarge,
    InvalidDesktopEntry,
    InvalidExec,
    MissingName,
    MissingExec,
    DuplicateDesktopId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexDiagnostic {
    pub path: PathBuf,
    pub kind: IndexDiagnosticKind,
    pub message: String,
}

/// Immutable generation returned to consumers. BTree collections make ordering
/// deterministic across rescans and distributions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IndexSnapshot {
    pub generation: u64,
    entries: BTreeMap<DesktopId, ApplicationRecord>,
    categories: BTreeMap<MainCategory, Vec<DesktopId>>,
    diagnostics: Vec<IndexDiagnostic>,
}

impl IndexSnapshot {
    pub(crate) fn from_parts(
        generation: u64,
        entries: BTreeMap<DesktopId, ApplicationRecord>,
        diagnostics: Vec<IndexDiagnostic>,
    ) -> Self {
        let mut categories: BTreeMap<MainCategory, Vec<DesktopId>> = BTreeMap::new();
        for record in entries.values().filter(|record| record.is_visible()) {
            let effective = if record.main_categories.is_empty() {
                vec![MainCategory::Other]
            } else {
                record.main_categories.clone()
            };
            for category in effective {
                categories
                    .entry(category)
                    .or_default()
                    .push(record.id.clone());
            }
        }
        for values in categories.values_mut() {
            values.sort_by(|left, right| {
                let left_name = entries
                    .get(left)
                    .map(|record| record.name.as_str())
                    .unwrap_or("");
                let right_name = entries
                    .get(right)
                    .map(|record| record.name.as_str())
                    .unwrap_or("");
                left_name
                    .to_lowercase()
                    .cmp(&right_name.to_lowercase())
                    .then_with(|| left.cmp(right))
            });
        }
        Self {
            generation,
            entries,
            categories,
            diagnostics,
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &ApplicationRecord> {
        self.entries.values()
    }

    pub fn visible_entries(&self) -> impl Iterator<Item = &ApplicationRecord> {
        self.entries.values().filter(|record| record.is_visible())
    }

    pub fn by_id(&self, id: &str) -> Option<&ApplicationRecord> {
        self.entries.get(&DesktopId::new(id.to_owned()))
    }

    pub fn category(&self, category: MainCategory) -> impl Iterator<Item = &ApplicationRecord> {
        self.categories
            .get(&category)
            .into_iter()
            .flatten()
            .filter_map(|id| self.entries.get(id))
    }

    pub fn diagnostics(&self) -> &[IndexDiagnostic] {
        &self.diagnostics
    }

    /// Lightweight common search for consumers. Finder-specific fuzzy ranking
    /// remains explicitly outside this stage.
    pub fn search(&self, query: &str) -> Vec<&ApplicationRecord> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return self.visible_entries().collect();
        }
        let mut matches: Vec<&ApplicationRecord> = self
            .visible_entries()
            .filter(|record| {
                record.name.to_lowercase().contains(&query)
                    || record.id.as_str().to_lowercase().contains(&query)
                    || record
                        .keywords
                        .iter()
                        .any(|keyword| keyword.to_lowercase().contains(&query))
                    || record
                        .categories
                        .iter()
                        .any(|category| category.to_lowercase().contains(&query))
            })
            .collect();
        matches.sort_by(|left, right| {
            let left_prefix = left.name.to_lowercase().starts_with(&query);
            let right_prefix = right.name.to_lowercase().starts_with(&query);
            right_prefix
                .cmp(&left_prefix)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
                .then_with(|| left.id.cmp(&right.id))
        });
        matches
    }

    pub(crate) fn with_generation(mut self, generation: u64) -> Self {
        self.generation = generation;
        self
    }

    pub(crate) fn entry_map(&self) -> &BTreeMap<DesktopId, ApplicationRecord> {
        &self.entries
    }
}

/// Precise change set broadcast after a successful live rescan.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IndexDelta {
    pub generation: u64,
    pub added: Vec<DesktopId>,
    pub removed: Vec<DesktopId>,
    pub changed: Vec<DesktopId>,
}

impl IndexDelta {
    pub(crate) fn between(previous: &IndexSnapshot, next: &IndexSnapshot) -> Self {
        let previous_ids: BTreeSet<_> = previous.entry_map().keys().cloned().collect();
        let next_ids: BTreeSet<_> = next.entry_map().keys().cloned().collect();
        let added = next_ids.difference(&previous_ids).cloned().collect();
        let removed = previous_ids.difference(&next_ids).cloned().collect();
        let changed = previous_ids
            .intersection(&next_ids)
            .filter(|id| previous.entry_map().get(*id) != next.entry_map().get(*id))
            .cloned()
            .collect();
        Self {
            generation: next.generation,
            added,
            removed,
            changed,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }
}
