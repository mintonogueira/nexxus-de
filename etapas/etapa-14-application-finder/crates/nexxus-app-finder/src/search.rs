//! Deterministic incremental search and ranking for the Application Finder.

use nexxus_xdg_application_index::{ApplicationRecord, IconReference, IndexSnapshot};

/// Metadata field that contributed the strongest part of a match.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchField {
    Name,
    Keyword,
    Comment,
    Category,
    DesktopId,
    Mixed,
}

/// Searchable projection owned by the Finder. It intentionally contains copied
/// presentation metadata so ranking never holds locks from the XDG index service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchDocument {
    pub desktop_id: String,
    pub name: String,
    pub icon: IconReference,
    pub keywords: Vec<String>,
    pub comment: Option<String>,
    pub categories: Vec<String>,
}

impl SearchDocument {
    /// Builds the Finder projection without reparsing the `.desktop` file.
    pub fn from_application(record: &ApplicationRecord, comment: Option<String>) -> Self {
        Self {
            desktop_id: record.id.as_str().to_owned(),
            name: record.name.clone(),
            icon: record.icon.clone(),
            keywords: record.keywords.clone(),
            comment,
            categories: record.categories.clone(),
        }
    }
}

/// Temporary inter-module seam for the `Comment` metadata missing from the
/// Stage 12 public record. It avoids violating module ownership by reparsing
/// desktop files in Stage 14.
pub trait CommentProvider {
    fn comment_for(&self, record: &ApplicationRecord) -> Option<String>;
}

/// Current Stage 12 compatibility provider. Search-by-comment is validated at
/// algorithm level, but full XDG integration remains blocked until Stage 12
/// exposes the localized `Comment` field.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoCommentProvider;

impl CommentProvider for NoCommentProvider {
    fn comment_for(&self, _record: &ApplicationRecord) -> Option<String> {
        None
    }
}

/// Immutable finder-side search corpus built from one XDG snapshot generation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FinderCorpus {
    generation: u64,
    documents: Vec<SearchDocument>,
}

impl FinderCorpus {
    /// Projects only entries that Stage 12 already considers visible.
    pub fn from_snapshot<P: CommentProvider>(snapshot: &IndexSnapshot, comments: &P) -> Self {
        let mut documents: Vec<_> = snapshot
            .visible_entries()
            .map(|record| {
                SearchDocument::from_application(record, comments.comment_for(record))
            })
            .collect();
        documents.sort_by(document_order);
        Self {
            generation: snapshot.generation,
            documents,
        }
    }

    /// Constructor used by tests and embedding adapters that already own fully
    /// normalized metadata.
    pub fn from_documents(generation: u64, mut documents: Vec<SearchDocument>) -> Self {
        documents.sort_by(document_order);
        Self {
            generation,
            documents,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn documents(&self) -> &[SearchDocument] {
        &self.documents
    }

    /// Performs AND matching across whitespace-separated query tokens and sorts
    /// by deterministic field-aware relevance.
    pub fn search(&self, query: &str) -> Vec<FinderMatch> {
        let normalized = normalize(query);
        if normalized.is_empty() {
            return self
                .documents
                .iter()
                .map(FinderMatch::unfiltered)
                .collect();
        }

        let tokens: Vec<&str> = normalized.split_whitespace().collect();
        let mut matches = Vec::new();
        for document in &self.documents {
            if let Some((score, field)) = score_document(document, &tokens) {
                matches.push(FinderMatch {
                    desktop_id: document.desktop_id.clone(),
                    name: document.name.clone(),
                    icon: document.icon.clone(),
                    score,
                    field,
                });
            }
        }
        matches.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| normalize(&left.name).cmp(&normalize(&right.name)))
                .then_with(|| left.desktop_id.cmp(&right.desktop_id))
        });
        matches
    }
}

/// Finder result detached from the source snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinderMatch {
    pub desktop_id: String,
    pub name: String,
    pub icon: IconReference,
    pub score: i32,
    pub field: MatchField,
}

impl FinderMatch {
    fn unfiltered(document: &SearchDocument) -> Self {
        Self {
            desktop_id: document.desktop_id.clone(),
            name: document.name.clone(),
            icon: document.icon.clone(),
            score: 0,
            field: MatchField::Name,
        }
    }
}

fn document_order(left: &SearchDocument, right: &SearchDocument) -> std::cmp::Ordering {
    normalize(&left.name)
        .cmp(&normalize(&right.name))
        .then_with(|| left.desktop_id.cmp(&right.desktop_id))
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn score_document(document: &SearchDocument, tokens: &[&str]) -> Option<(i32, MatchField)> {
    let name = normalize(&document.name);
    let desktop_id = normalize(&document.desktop_id);
    let keywords: Vec<String> = document.keywords.iter().map(|value| normalize(value)).collect();
    let categories: Vec<String> = document.categories.iter().map(|value| normalize(value)).collect();
    let comment = document.comment.as_deref().map(normalize);

    let mut total = 0i32;
    let mut strongest = None;
    for token in tokens {
        let candidates = [
            score_name(&name, token).map(|score| (score, MatchField::Name)),
            score_collection(&keywords, token, 720, 650, 590)
                .map(|score| (score, MatchField::Keyword)),
            comment
                .as_deref()
                .and_then(|value| score_text(value, token, 540, 500, 460))
                .map(|score| (score, MatchField::Comment)),
            score_collection(&categories, token, 440, 400, 360)
                .map(|score| (score, MatchField::Category)),
            score_text(&desktop_id, token, 300, 270, 240)
                .map(|score| (score, MatchField::DesktopId)),
        ];

        let best = candidates.into_iter().flatten().max_by_key(|candidate| candidate.0)?;
        total = total.saturating_add(best.0);
        strongest = match strongest {
            None => Some(best.1),
            Some(current) if current == best.1 => Some(current),
            Some(_) => Some(MatchField::Mixed),
        };
    }

    total = total.saturating_sub(name.chars().count().min(80) as i32);
    Some((total, strongest.unwrap_or(MatchField::Name)))
}

fn score_name(value: &str, token: &str) -> Option<i32> {
    if value == token {
        return Some(1000);
    }
    if value.starts_with(token) {
        return Some(920);
    }
    if value
        .split(|character: char| !character.is_alphanumeric())
        .any(|word| word.starts_with(token))
    {
        return Some(860);
    }
    if value.contains(token) {
        return Some(790);
    }
    is_subsequence(token, value).then_some(330)
}

fn score_collection(
    values: &[String],
    token: &str,
    exact: i32,
    prefix: i32,
    contains: i32,
) -> Option<i32> {
    values
        .iter()
        .filter_map(|value| score_text(value, token, exact, prefix, contains))
        .max()
}

fn score_text(
    value: &str,
    token: &str,
    exact: i32,
    prefix: i32,
    contains: i32,
) -> Option<i32> {
    if value == token {
        Some(exact)
    } else if value.starts_with(token) {
        Some(prefix)
    } else if value.contains(token) {
        Some(contains)
    } else {
        None
    }
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    if needle.chars().count() < 2 {
        return false;
    }
    let mut expected = needle.chars();
    let mut current = expected.next();
    for character in haystack.chars() {
        if current == Some(character) {
            current = expected.next();
            if current.is_none() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon() -> IconReference {
        IconReference::ExternalName("demo".to_owned())
    }

    fn doc(
        id: &str,
        name: &str,
        keywords: &[&str],
        comment: Option<&str>,
        categories: &[&str],
    ) -> SearchDocument {
        SearchDocument {
            desktop_id: id.to_owned(),
            name: name.to_owned(),
            icon: icon(),
            keywords: keywords.iter().map(|value| (*value).to_owned()).collect(),
            comment: comment.map(str::to_owned),
            categories: categories.iter().map(|value| (*value).to_owned()).collect(),
        }
    }

    fn corpus() -> FinderCorpus {
        FinderCorpus::from_documents(
            7,
            vec![
                doc(
                    "org.demo.Terminal.desktop",
                    "Nexxus Terminal",
                    &["shell", "console"],
                    Some("Emulador de terminal rápido"),
                    &["System", "Utility"],
                ),
                doc(
                    "org.demo.Editor.desktop",
                    "Editor",
                    &["texto", "code"],
                    Some("Editor de documentos"),
                    &["Utility"],
                ),
                doc(
                    "org.demo.Camera.desktop",
                    "Câmera",
                    &["foto"],
                    Some("Captura de vídeo"),
                    &["AudioVideo"],
                ),
            ],
        )
    }

    #[test]
    fn empty_query_returns_deterministic_name_order() {
        let result = corpus().search("");
        let names: Vec<_> = result.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, ["Câmera", "Editor", "Nexxus Terminal"]);
    }

    #[test]
    fn exact_and_prefix_name_matches_outrank_weaker_fields() {
        let corpus = corpus();
        let exact = corpus.search("editor");
        assert_eq!(exact[0].desktop_id, "org.demo.Editor.desktop");
        assert_eq!(exact[0].field, MatchField::Name);

        let prefix = corpus.search("nex");
        assert_eq!(prefix[0].desktop_id, "org.demo.Terminal.desktop");
    }

    #[test]
    fn searches_keywords_comments_and_categories() {
        let corpus = corpus();
        assert_eq!(corpus.search("console")[0].field, MatchField::Keyword);
        assert_eq!(corpus.search("documentos")[0].field, MatchField::Comment);
        assert_eq!(corpus.search("audiovideo")[0].field, MatchField::Category);
    }

    #[test]
    fn supports_unicode_case_folding_from_rust_lowercase() {
        let result = corpus().search("CÂM");
        assert_eq!(result[0].desktop_id, "org.demo.Camera.desktop");
    }

    #[test]
    fn fuzzy_subsequence_is_lower_priority_but_available() {
        let result = corpus().search("nxt");
        assert_eq!(result[0].desktop_id, "org.demo.Terminal.desktop");
        assert!(result[0].score < 500);
    }

    #[test]
    fn all_query_tokens_must_match() {
        let result = corpus().search("terminal console");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, MatchField::Mixed);
    }
}
