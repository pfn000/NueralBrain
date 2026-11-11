use crate::corpus::DocumentId;
use crate::index::InvertedIndex;

#[derive(Debug, Clone)]
pub struct RankerConfig {
    pub k1: f32,
    pub b: f32,
    pub concept_tag_boost: f32,
}

impl Default for RankerConfig {
    fn default() -> Self {
        Self { k1: 1.4, b: 0.75, concept_tag_boost: 1.6 }
    }
}

/// BM25-lite score for one term, returned in stable document-ID order.
pub fn score_bm25lite(
    index: &InvertedIndex,
    term: &str,
    config: &RankerConfig,
) -> Vec<(DocumentId, f32)> {
    let Some(postings) = index.postings(term) else { return Vec::new(); };
    if postings.is_empty() { return Vec::new(); }

    let document_count = index.total_docs().max(1) as f32;
    let document_frequency = postings.len() as f32;
    let inverse_frequency = (
        (document_count - document_frequency + 0.5) / (document_frequency + 0.5) + 1.0
    ).ln().max(0.0);
    let average_length = index.avg_doc_len().max(1.0);

    postings.iter().map(|(document_id, term_frequency)| {
        let length = index.doc_len(*document_id).max(1) as f32;
        let tf = *term_frequency as f32;
        let numerator = tf * (config.k1 + 1.0);
        let denominator = tf + config.k1 * (1.0 - config.b + config.b * length / average_length);
        (*document_id, inverse_frequency * numerator / denominator)
    }).collect()
}
