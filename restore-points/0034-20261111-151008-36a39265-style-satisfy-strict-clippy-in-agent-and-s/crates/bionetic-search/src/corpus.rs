use std::collections::HashMap;

#[cfg(feature = "corpus-json")]
use std::path::Path;

use crate::concept::{Concept, ConceptId};
use crate::error::SearchResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "corpus-json", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentId(pub u32);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "corpus-json", derive(serde::Serialize, serde::Deserialize))]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub body: String,
    pub concepts: Vec<ConceptId>,
    pub source_uri: Option<String>,
}

/// Owned, editable source of indexed concepts and documents.
#[derive(Debug, Default)]
pub struct CustomCorpus {
    concepts: Vec<Concept>,
    documents: Vec<Document>,
    label_to_concept: HashMap<String, ConceptId>,
}

impl CustomCorpus {
    pub fn new() -> Self { Self::default() }

    pub fn add_concept(&mut self, mut concept: Concept) -> ConceptId {
        concept.id = ConceptId(self.concepts.len() as u32);
        let id = concept.id;
        self.label_to_concept.insert(normalize(&concept.label), id);
        for form in &concept.surface_forms {
            self.label_to_concept.insert(normalize(form), id);
        }
        self.concepts.push(concept);
        id
    }

    pub fn add_document(&mut self, mut document: Document) -> DocumentId {
        document.id = DocumentId(self.documents.len() as u32);
        let id = document.id;
        self.documents.push(document);
        id
    }

    pub fn concept(&self, id: ConceptId) -> Option<&Concept> {
        self.concepts.get(id.0 as usize)
    }

    pub fn concept_by_label(&self, label: &str) -> Option<&Concept> {
        let id = self.label_to_concept.get(&normalize(label))?;
        self.concept(*id)
    }

    pub fn document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(id.0 as usize)
    }

    pub fn concepts(&self) -> &[Concept] { &self.concepts }
    pub fn documents(&self) -> &[Document] { &self.documents }
    pub fn len_concepts(&self) -> usize { self.concepts.len() }
    pub fn len_documents(&self) -> usize { self.documents.len() }

    #[cfg(feature = "corpus-json")]
    pub fn load_json(path: impl AsRef<Path>) -> SearchResult<Self> {
        let raw: CorpusFile = serde_json::from_slice(&std::fs::read(path)?)?;
        let mut corpus = Self::new();
        for concept in raw.concepts { corpus.add_concept(concept); }
        for document in raw.documents { corpus.add_document(document); }
        Ok(corpus)
    }

    #[cfg(feature = "corpus-json")]
    pub fn save_json(&self, path: impl AsRef<Path>) -> SearchResult<()> {
        let file = CorpusFile {
            concepts: self.concepts.clone(),
            documents: self.documents.clone(),
        };
        std::fs::write(path, serde_json::to_vec_pretty(&file)?)?;
        Ok(())
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(feature = "corpus-json")]
#[derive(serde::Serialize, serde::Deserialize)]
struct CorpusFile {
    concepts: Vec<Concept>,
    documents: Vec<Document>,
}

/// Tiny seed set for demos and deterministic tests.
pub fn seed_corpus() -> CustomCorpus {
    let mut corpus = CustomCorpus::new();
    let brain = corpus.add_concept(
        Concept::new(0, "brain", (0, 16))
            .with_surface_forms(["brains", "neural", "neuron", "neurons"]),
    );
    let sleep = corpus.add_concept(
        Concept::new(1, "sleep", (16, 32))
            .with_surface_forms(["sleeping", "dream", "dreams", "dreaming"]),
    );
    let vision = corpus.add_concept(
        Concept::new(2, "vision", (32, 48))
            .with_surface_forms(["sight", "see", "visual", "eye", "eyes"]),
    );

    corpus.add_document(Document {
        id: DocumentId(0),
        title: "Why brains sleep".into(),
        body: "Brains consolidate memories during sleep. Dreams replay neural patterns.".into(),
        concepts: vec![brain, sleep],
        source_uri: None,
    });
    corpus.add_document(Document {
        id: DocumentId(0),
        title: "Vision cones".into(),
        body: "Vision begins at the cones. Three cone types sense red, green, and blue light.".into(),
        concepts: vec![vision],
        source_uri: None,
    });
    corpus.add_document(Document {
        id: DocumentId(0),
        title: "Dreaming and vision".into(),
        body: "During dreaming, visual cortex reactivates. The brain replays sights.".into(),
        concepts: vec![brain, sleep, vision],
        source_uri: None,
    });
    corpus
}
