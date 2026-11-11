use std::sync::Arc;
use parking_lot::Mutex;
use bionetic_agent::{AgentBuilder, PermissionPolicy};
use bionetic_core::MOTOR;
use bionetic_sdk::BrainBuilder;
use bionetic_search::{
    concept::ConceptId,
    corpus::seed_corpus,
    encoder::{SearchEncoder, SearchEncoderConfig},
    effector::SearchEffector,
    engine::{SearchEngine, SearchEngineConfig},
    query::QueryBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut brain = BrainBuilder::new().with_corpora(bionetic_core::CORPUS_COUNT_TOTAL).with_dots_per_corpus(128).build();
    let search_engine = Arc::new(Mutex::new(SearchEngine::new(
        seed_corpus(), SearchEngineConfig::default(),
    )));
    let slots = vec![ConceptId(0), ConceptId(1), ConceptId(2)];
    let effector = SearchEffector::new(search_engine.clone(), slots);

    // Local retrieval requires no network capability; deny-all remains the default.
    let mut agent = AgentBuilder::new()
        .with_effector(effector)
        .policy(PermissionPolicy::deny_all())
        .cooldown_ticks(0)
        .build()?;

    // Deterministic demo stimulus: fire the Query region dots directly.
    let tick = brain.engine().tick;
    if let Some(corpus) = brain.engine_mut().corpora.get_mut(&MOTOR) {
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        for id in ids.into_iter().take(16) {
            if let Some(dot) = corpus.dots.get_mut(&id) { dot.fire(tick); }
        }
    }
    let report = agent.tick(brain.engine_mut())?;
    println!("agent executed={} blocked={}", report.executed, report.blocked);

    // Encode a second explicit query as targeted sensory pulses.
    let guard = search_engine.lock();
    let result = guard.search(&QueryBuilder::new().concept(ConceptId(0)).build())?;
    let encoder = SearchEncoder::new(SearchEncoderConfig::default());
    let queued = encoder.encode(&result, guard.corpus(), brain.engine_mut());
    drop(guard);
    brain.engine_mut().tick();
    println!("queued {queued} search-feedback pulses; engine tick advanced");
    Ok(())
}
