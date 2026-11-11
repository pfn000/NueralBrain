use bionetic_core::ids::{CorpusId, DotId};
use bionetic_core::SENSORY_IN;
use bionetic_engine::engine::BioneticEngine;

use crate::concept::ConceptId;
use crate::corpus::CustomCorpus;
use crate::result::SearchResult;
use crate::{DEFAULT_TOP_K, DOTS_PER_HIT};

#[derive(Debug, Clone)]
pub struct SearchEncoderConfig {
    pub target_corpus: CorpusId,
    pub amplitude: f32,
    pub top_k: usize,
    pub stimulate_concept_dots: bool,
}

impl Default for SearchEncoderConfig {
    fn default() -> Self {
        Self {
            target_corpus: SENSORY_IN,
            amplitude: 0.6,
            top_k: DEFAULT_TOP_K,
            stimulate_concept_dots: true,
        }
    }
}

pub struct SearchEncoder {
    config: SearchEncoderConfig,
}

impl SearchEncoder {
    pub fn new(config: SearchEncoderConfig) -> Self { Self { config } }

    /// Queue deterministic, rank-weighted pulses in the selected engine corpus.
    /// The engine delivers them on its next tick; the return value is pulses queued.
    pub fn encode(
        &self,
        result: &SearchResult,
        search_corpus: &CustomCorpus,
        engine: &mut BioneticEngine,
    ) -> usize {
        let Some(corpus) = engine.corpora.get(&self.config.target_corpus) else { return 0; };
        let mut dot_ids: Vec<DotId> = corpus.dots.keys().copied().collect();
        dot_ids.sort_unstable();
        if dot_ids.is_empty() { return 0; }

        let mut queued = 0usize;
        for (rank, hit) in result.hits.iter().take(self.config.top_k).enumerate() {
            let decay = 1.0 / (1.0 + rank as f32);
            let amplitude = (self.config.amplitude * decay).clamp(0.0, 1.0) as f64;
            let base = ((hit.doc_id.0 as usize % dot_ids.len()) * DOTS_PER_HIT) % dot_ids.len();

            for offset in 0..DOTS_PER_HIT.min(dot_ids.len()) {
                let dot_id = dot_ids[(base + offset) % dot_ids.len()];
                engine.inject_pulse(dot_id, self.config.target_corpus, amplitude, 0);
                queued += 1;
            }

            if self.config.stimulate_concept_dots {
                for concept_id in &hit.matched_concepts {
                    queued += self.stimulate_concept_range(
                        engine, &dot_ids, search_corpus, *concept_id, amplitude,
                    );
                }
            }
        }
        queued
    }

    fn stimulate_concept_range(
        &self,
        engine: &mut BioneticEngine,
        dot_ids: &[DotId],
        corpus: &CustomCorpus,
        concept_id: ConceptId,
        amplitude: f64,
    ) -> usize {
        let Some(concept) = corpus.concept(concept_id) else { return 0; };
        let start = concept.dot_range.0.min(dot_ids.len());
        let end = concept.dot_range.1.min(dot_ids.len());
        let mut queued = 0;
        for dot_id in dot_ids.iter().take(end).skip(start).copied() {
            engine.inject_pulse(dot_id, self.config.target_corpus, amplitude, 0);
            queued += 1;
        }
        queued
    }
}
