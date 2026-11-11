use crate::concept::ConceptId;
use crate::corpus::DocumentId;

#[derive(Debug, Clone)]
pub struct Hit {
    pub doc_id: DocumentId,
    pub score: f32,
    pub matched_concepts: Vec<ConceptId>,
    pub title: String,
    pub snippet: String,
    pub source_uri: Option<String>,
    pub from_web: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SearchResult {
    pub hits: Vec<Hit>,
    pub total_candidates: usize,
    pub latency_us: u64,
}

impl SearchResult {
    pub fn is_empty(&self) -> bool { self.hits.is_empty() }
    pub fn best(&self) -> Option<&Hit> { self.hits.first() }
}
