use bionetic_agent::{AgentBuilder, IntentKind, NoopEffector, PermissionPolicy};
use bionetic_sdk::BrainBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(NoopEffector::new("noop-query").only(IntentKind::Query))
        .policy(PermissionPolicy::deny_all())
        .cooldown_ticks(0)
        .build()?;

    // Synthetic firing makes the demo deterministic; production activity comes from engine dynamics.
    let tick = brain.engine().tick;
    if let Some(corpus) = brain.engine_mut().corpora.get_mut(&bionetic_core::ids::CorpusId(5)) {
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        for id in ids.into_iter().take(16) {
            if let Some(dot) = corpus.dots.get_mut(&id) { dot.fire(tick); }
        }
    }
    let report = agent.tick(brain.engine_mut())?;
    println!("intent={:?} executed={} blocked={}", report.intent.kind, report.executed, report.blocked);
    Ok(())
}
