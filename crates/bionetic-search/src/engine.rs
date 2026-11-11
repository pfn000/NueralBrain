use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use crate::concept::ConceptId;
use crate::corpus::{CustomCorpus, Document, DocumentId};
use crate::error::{SearchError, SearchResult as SearchOutcome};
use crate::index::{tokens, InvertedIndex};
use crate::query::Query;
use crate::ranker::{score_bm25lite, RankerConfig};
use crate::result::{Hit, SearchResult};

#[cfg(feature = "web")]
use crate::web::{extract_text, WebFetcher};

#[derive(Debug, Clone)]
pub struct SearchEngineConfig {
    pub ranker: RankerConfig,
    pub snippet_chars: usize,
    pub default_limit: usize,
    #[cfg(feature = "web")]
    pub web_enabled_by_default: bool,
}

impl Default for SearchEngineConfig {
    fn default() -> Self {
        Self {
            ranker: RankerConfig::default(),
            snippet_chars: 180,
            default_limit: crate::DEFAULT_TOP_K,
            #[cfg(feature = "web")]
            web_enabled_by_default: false,
        }
    }
}

pub struct SearchEngine {
    corpus: CustomCorpus,
    index: InvertedIndex,
    config: SearchEngineConfig,
    #[cfg(feature = "web")]
    web: Option<WebFetcher>,
    #[cfg(feature = "web")]
    web_slots: Vec<String>,
}

impl SearchEngine {
    pub fn new(corpus: CustomCorpus, config: SearchEngineConfig) -> Self {
        let index = InvertedIndex::build(&corpus);
        Self {
            corpus,
            index,
            config,
            #[cfg(feature = "web")]
            web: None,
            #[cfg(feature = "web")]
            web_slots: Vec::new(),
        }
    }

    pub fn corpus(&self) -> &CustomCorpus { &self.corpus }
    pub fn index(&self) -> &InvertedIndex { &self.index }

    #[cfg(feature = "web")]
    pub fn with_web(mut self, fetcher: WebFetcher, slots: Vec<String>) -> Self {
        self.web = Some(fetcher);
        self.web_slots = slots;
        self
    }

    /// Rebuild after external corpus changes.
    pub fn reindex(&mut self) {
        self.index = InvertedIndex::build(&self.corpus);
    }

    pub fn search(&self, query: &Query) -> SearchOutcome<SearchResult> {
        if query.concepts.is_empty() && query.text_hints.is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        let started = Instant::now();

        let mut term_set = BTreeSet::new();
        for concept_id in &query.concepts {
            let concept = self.corpus.concept(*concept_id)
                .ok_or(SearchError::UnknownConcept(concept_id.0))?;
            term_set.insert(concept.label.to_lowercase());
            for form in &concept.surface_forms {
                term_set.insert(form.to_lowercase());
            }
        }
        for hint in &query.text_hints {
            for token in tokens(hint) {
                if self.corpus.concept_by_label(&token).is_some() {
                    term_set.insert(token);
                }
            }
        }
        if term_set.is_empty() {
            return Ok(SearchResult {
                hits: Vec::new(),
                total_candidates: 0,
                latency_us: started.elapsed().as_micros() as u64,
            });
        }

        let mut scores: HashMap<DocumentId, f32> = HashMap::new();
        for term in &term_set {
            for (document_id, score) in score_bm25lite(&self.index, term, &self.config.ranker) {
                *scores.entry(document_id).or_insert(0.0) += score;
            }
        }

        for (document_id, score) in scores.iter_mut() {
            if let Some(document) = self.corpus.document(*document_id) {
                let overlap = document.concepts.iter()
                    .filter(|concept_id| query.concepts.contains(concept_id))
                    .count();
                if overlap > 0 {
                    *score *= self.config.ranker.concept_tag_boost.powi(overlap as i32);
                }
            }
        }

        let total_candidates = scores.len();
        let limit = if query.limit == 0 { self.config.default_limit.max(1) } else { query.limit };
        let mut ranked: Vec<_> = scores.into_iter().collect();
        ranked.sort_by(|left, right| {
            right.1.total_cmp(&left.1).then_with(|| left.0.0.cmp(&right.0.0))
        });
        ranked.truncate(limit.max(1));

        let hits: Vec<Hit> = ranked.into_iter().filter_map(|(id, score)| {
            let document = self.corpus.document(id)?;
            Some(hit_from_document(
                document,
                score,
                query.concepts.as_slice(),
                self.config.snippet_chars,
                false,
            ))
        }).collect();
        #[cfg(feature = "web")]
        let mut hits = hits;

        let include_web = query.web
            || {
                #[cfg(feature = "web")]
                { self.config.web_enabled_by_default }
                #[cfg(not(feature = "web"))]
                { false }
            };

        if include_web {
            #[cfg(feature = "web")]
            {
                let fetcher = self.web.as_ref()
                    .ok_or_else(|| SearchError::Web("web was requested but no WebFetcher is configured".into()))?;
                let mut attempted = 0usize;
                for (slot_index, address) in self.web_slots.iter().enumerate() {
                    let slot_selected = query.concepts.iter().any(|id| id.0 as usize == slot_index);
                    if !slot_selected { continue; }
                    attempted += 1;
                    if let Ok(html) = fetcher.fetch(address) {
                        let (title, body) = extract_text(&html);
                        hits.push(Hit {
                            doc_id: DocumentId(u32::MAX.saturating_sub(slot_index as u32)),
                            score: 0.5,
                            matched_concepts: query.concepts.clone(),
                            title,
                            snippet: make_snippet(&body, self.config.snippet_chars),
                            source_uri: Some(address.clone()),
                            from_web: true,
                        });
                    }
                }
                if attempted == 0 {
                    return Err(SearchError::Web("no registered web slot matched the selected concepts".into()));
                }
            }
            #[cfg(not(feature = "web"))]
            {
                return Err(SearchError::Config("web search requested, but the crate was built without the web feature".into()));
            }
        }

        Ok(SearchResult {
            hits,
            total_candidates,
            latency_us: started.elapsed().as_micros() as u64,
        })
    }

    pub fn corpus_mut(&mut self) -> &mut CustomCorpus { &mut self.corpus }
}

fn hit_from_document(
    document: &Document,
    score: f32,
    query_concepts: &[ConceptId],
    snippet_chars: usize,
    from_web: bool,
) -> Hit {
    Hit {
        doc_id: document.id,
        score,
        matched_concepts: document.concepts.iter()
            .filter(|id| query_concepts.contains(id))
            .copied()
            .collect(),
        title: document.title.clone(),
        snippet: make_snippet(&document.body, snippet_chars),
        source_uri: document.source_uri.clone(),
        from_web,
    }
}

fn make_snippet(body: &str, max_chars: usize) -> String {
    let mut snippet: String = body.chars().take(max_chars).collect();
    if body.chars().count() > max_chars { snippet.push('…'); }
    snippet
}
