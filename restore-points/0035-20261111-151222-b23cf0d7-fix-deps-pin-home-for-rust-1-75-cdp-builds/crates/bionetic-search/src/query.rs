use crate::concept::ConceptId;

/// A bounded query composed of known concept IDs and optional host-provided hints.
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub concepts: Vec<ConceptId>,
    pub text_hints: Vec<String>,
    pub limit: usize,
    pub web: bool,
}

#[derive(Debug, Default)]
pub struct QueryBuilder {
    inner: Query,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self { inner: Query { limit: crate::DEFAULT_TOP_K, ..Query::default() } }
    }

    pub fn concept(mut self, id: ConceptId) -> Self {
        self.inner.concepts.push(id);
        self
    }

    pub fn concepts<I: IntoIterator<Item = ConceptId>>(mut self, ids: I) -> Self {
        self.inner.concepts.extend(ids);
        self
    }

    /// Add a host-provided hint; only tokens already known as concept labels or aliases survive.
    pub fn text(mut self, hint: impl Into<String>) -> Self {
        self.inner.text_hints.push(hint.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.inner.limit = limit.max(1);
        self
    }

    pub fn web(mut self, enabled: bool) -> Self {
        self.inner.web = enabled;
        self
    }

    pub fn build(self) -> Query { self.inner }
}
