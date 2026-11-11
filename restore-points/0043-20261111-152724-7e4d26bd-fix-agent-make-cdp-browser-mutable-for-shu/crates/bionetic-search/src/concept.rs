/// Stable concept slot identifier assigned by CustomCorpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "corpus-json", derive(serde::Serialize, serde::Deserialize))]
pub struct ConceptId(pub u32);

/// Symbolic anchor that can map to a document cluster and sensory dot range.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "corpus-json", derive(serde::Serialize, serde::Deserialize))]
pub struct Concept {
    pub id: ConceptId,
    pub label: String,
    pub surface_forms: Vec<String>,
    pub dot_range: (usize, usize),
    pub tags: Vec<String>,
}

impl Concept {
    pub fn new(id: u32, label: impl Into<String>, dot_range: (usize, usize)) -> Self {
        Self {
            id: ConceptId(id),
            label: label.into(),
            surface_forms: Vec::new(),
            dot_range,
            tags: Vec::new(),
        }
    }

    pub fn with_surface_forms<I, S>(mut self, forms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.surface_forms = forms.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }
}
