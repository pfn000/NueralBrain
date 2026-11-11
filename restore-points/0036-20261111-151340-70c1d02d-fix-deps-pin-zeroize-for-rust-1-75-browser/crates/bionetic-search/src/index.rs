use std::collections::HashMap;

use crate::corpus::{CustomCorpus, DocumentId};

/// Lowercase token -> document postings and term frequency.
#[derive(Debug, Default)]
pub struct InvertedIndex {
    postings: HashMap<String, Vec<(DocumentId, u32)>>,
    doc_len: HashMap<DocumentId, u32>,
    avg_doc_len: f32,
    total_docs: u32,
}

impl InvertedIndex {
    pub fn build(corpus: &CustomCorpus) -> Self {
        let mut index = Self::default();
        let mut total_len = 0u64;

        for document in corpus.documents() {
            let mut counts: HashMap<String, u32> = HashMap::new();
            let mut length = 0u32;
            for token in tokens(&format!("{} {}", document.title, document.body)) {
                *counts.entry(token).or_insert(0) += 1;
                length = length.saturating_add(1);
            }
            index.doc_len.insert(document.id, length);
            total_len += u64::from(length);
            for (term, frequency) in counts {
                index.postings.entry(term).or_default().push((document.id, frequency));
            }
        }

        index.total_docs = corpus.documents().len() as u32;
        index.avg_doc_len = if index.total_docs == 0 {
            0.0
        } else {
            total_len as f32 / index.total_docs as f32
        };
        for postings in index.postings.values_mut() {
            postings.sort_by_key(|(document_id, _)| document_id.0);
        }
        index
    }

    pub fn postings(&self, term: &str) -> Option<&[(DocumentId, u32)]> {
        self.postings.get(&term.to_lowercase()).map(Vec::as_slice)
    }

    pub fn doc_len(&self, id: DocumentId) -> u32 {
        self.doc_len.get(&id).copied().unwrap_or(0)
    }

    pub fn avg_doc_len(&self) -> f32 { self.avg_doc_len }
    pub fn total_docs(&self) -> u32 { self.total_docs }
}

pub(crate) fn tokens(value: &str) -> impl Iterator<Item = String> + '_ {
    value.split(|character: char| !character.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
}
