use bionetic_agent::{
    ActionOutcome, AgentBuilder, Capability, Effector, EffectorId, Intent, IntentKind,
    PermissionPolicy,
};
use bionetic_agent::error::{AgentError, AgentResult};
use bionetic_core::ids::CorpusId;
use bionetic_sdk::BrainBuilder;

struct NeedsNetwork;
impl Effector for NeedsNetwork {
    fn id(&self) -> EffectorId { EffectorId::new("needs-net") }
    fn handles(&self, intent: &Intent) -> bool { matches!(intent.kind, IntentKind::Query | IntentKind::Act) }
    fn required_capabilities(&self, _intent: &Intent) -> Vec<Capability> {
        vec![Capability::NetworkHttp]
    }
    fn execute(&self, _intent: &Intent) -> AgentResult<ActionOutcome> {
        Ok(ActionOutcome::success(b"ran".to_vec(), 100))
    }
}

fn stimulate(engine: &mut bionetic_engine::engine::BioneticEngine, offset: usize, size: usize) {
    let tick = engine.tick;
    if let Some(corpus) = engine.corpora.get_mut(&CorpusId(5)) {
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        for id in ids.into_iter().skip(offset).take(size) {
            if let Some(dot) = corpus.dots.get_mut(&id) { dot.fire(tick); }
        }
    }
}

#[test]
fn deny_all_blocks_effector_that_needs_network() {
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(NeedsNetwork)
        .policy(PermissionPolicy::deny_all())
        .cooldown_ticks(0)
        .build().unwrap();
    stimulate(brain.engine_mut(), 0, 16);
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert!(report.blocked);
    assert!(!report.executed);
}

#[test]
fn granting_network_capability_allows_execution() {
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(NeedsNetwork)
        .policy(PermissionPolicy::deny_all().grant(Capability::NetworkHttp))
        .cooldown_ticks(0)
        .build().unwrap();
    stimulate(brain.engine_mut(), 0, 16);
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert!(report.executed);
    assert!(!report.blocked);
    assert_eq!(report.effector_id.as_deref(), Some("needs-net"));
}

#[test]
fn act_is_denied_in_read_only_mode() {
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(NeedsNetwork)
        .policy(PermissionPolicy::allow_all().read_only(true))
        .cooldown_ticks(0)
        .build().unwrap();
    stimulate(brain.engine_mut(), 16, 16);
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert!(report.blocked);
}

#[test]
fn unknown_intent_has_no_effector() {
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(NeedsNetwork)
        .policy(PermissionPolicy::allow_all())
        .cooldown_ticks(0)
        .build().unwrap();
    stimulate(brain.engine_mut(), 48, 16);
    let error = agent.tick(brain.engine_mut()).unwrap_err();
    assert!(matches!(error, AgentError::NoEffector(IntentKind::Emit)));
}

#[test]
fn builder_rejects_empty_registry() {
    assert!(matches!(AgentBuilder::new().build(), Err(AgentError::Config(_))));
}

#[test]
fn audit_records_blocked_attempts() {
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(NeedsNetwork)
        .policy(PermissionPolicy::deny_all())
        .cooldown_ticks(0)
        .build().unwrap();
    stimulate(brain.engine_mut(), 0, 16);
    let _ = agent.tick(brain.engine_mut()).unwrap();
    assert_eq!(agent.audit().len(), 1);
}
