use bionetic_search::{
    concept::{Concept, ConceptId},
    corpus::{seed_corpus, CustomCorpus, Document, DocumentId},
    encoder::{SearchEncoder, SearchEncoderConfig},
    engine::{SearchEngine, SearchEngineConfig},
    query::QueryBuilder,
};
use bionetic_sdk::BrainBuilder;
use bionetic_core::ids::CorpusId;

#[test]
fn concept_query_returns_tagged_documents() {
    let engine = SearchEngine::new(seed_corpus(), SearchEngineConfig::default());
    let result = engine.search(&QueryBuilder::new().concept(ConceptId(1)).build()).unwrap();
    assert!(!result.hits.is_empty());
    assert!(result.hits.iter().any(|hit| hit.matched_concepts.contains(&ConceptId(1))));
}

#[test]
fn text_hints_only_match_known_surface_forms() {
    let engine = SearchEngine::new(seed_corpus(), SearchEngineConfig::default());
    let result = engine.search(&QueryBuilder::new().text("quantum chromodynamics").build()).unwrap();
    assert!(result.hits.is_empty());
}

#[test]
fn empty_query_is_rejected() {
    let engine = SearchEngine::new(seed_corpus(), SearchEngineConfig::default());
    assert!(engine.search(&QueryBuilder::new().build()).is_err());
}

#[test]
fn concept_tag_boost_moves_tagged_document_ahead() {
    let mut corpus = CustomCorpus::new();
    let alpha = corpus.add_concept(Concept::new(0, "alpha", (0, 8)).with_surface_forms(["alpha"]));
    corpus.add_document(Document {
        id: DocumentId(0), title: "alpha topic".into(), body: "alpha".into(),
        concepts: vec![alpha], source_uri: None,
    });
    corpus.add_document(Document {
        id: DocumentId(0), title: "alpha mention".into(), body: "alpha alpha alpha alpha".into(),
        concepts: vec![], source_uri: None,
    });
    let result = SearchEngine::new(corpus, SearchEngineConfig::default())
        .search(&QueryBuilder::new().concept(alpha).build()).unwrap();
    assert_eq!(result.hits[0].title, "alpha topic");
}

#[cfg(feature = "corpus-json")]
#[test]
fn corpus_json_roundtrip_rebuilds_lookup_index() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("corpus.json");
    let before = seed_corpus();
    before.save_json(&path).unwrap();
    let after = CustomCorpus::load_json(&path).unwrap();
    assert_eq!(before.len_concepts(), after.len_concepts());
    assert_eq!(before.len_documents(), after.len_documents());
    assert!(after.concept_by_label("neural").is_some());
}

#[test]
fn encoder_queues_pulses_using_real_engine_dot_ids() {
    let corpus = seed_corpus();
    let engine = SearchEngine::new(seed_corpus(), SearchEngineConfig::default());
    let result = engine.search(&QueryBuilder::new().concept(ConceptId(0)).build()).unwrap();
    let encoder = SearchEncoder::new(SearchEncoderConfig::default());
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(64).build();
    let queued = encoder.encode(&result, &corpus, brain.engine_mut());
    assert!(queued > 0);
    assert!(brain.engine().pending_pulses() >= queued);
    assert!(brain.engine().has_corpus(CorpusId(0)));
}

#[cfg(feature = "agent")]
#[test]
fn local_search_effector_runs_without_network_capability() {
    use std::sync::Arc;
    use parking_lot::Mutex;
    use bionetic_agent::{AgentBuilder, PermissionPolicy};
    use bionetic_search::effector::SearchEffector;

    let search = Arc::new(Mutex::new(SearchEngine::new(seed_corpus(), SearchEngineConfig::default())));
    let effector = SearchEffector::new(search, vec![ConceptId(0), ConceptId(1), ConceptId(2)]);
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(effector)
        .policy(PermissionPolicy::deny_all())
        .cooldown_ticks(0)
        .build().unwrap();

    let tick = brain.engine().tick;
    if let Some(corpus) = brain.engine_mut().corpora.get_mut(&CorpusId(5)) {
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        for id in ids.into_iter().take(16) {
            if let Some(dot) = corpus.dots.get_mut(&id) { dot.fire(tick); }
        }
    }
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert!(report.executed);
    assert!(!report.blocked);
}
