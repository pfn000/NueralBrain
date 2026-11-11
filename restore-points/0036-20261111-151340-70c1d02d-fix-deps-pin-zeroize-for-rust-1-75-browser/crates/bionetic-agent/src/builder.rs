use bionetic_core::ids::CorpusId;

use crate::audit::AuditLog;
use crate::effector::Effector;
use crate::error::{AgentError, AgentResult};
use crate::intent::IntentDecoder;
use crate::loop_::AgentLoop;
use crate::policy::PermissionPolicy;
use crate::registry::EffectorRegistry;
use crate::INTENT_FIRE_THRESHOLD;

pub struct AgentBuilder {
    motor_corpus: CorpusId,
    threshold: f32,
    policy: PermissionPolicy,
    registry: EffectorRegistry,
    audit_capacity: usize,
    cooldown_ticks: u64,
    feedback: bool,
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self {
            motor_corpus: CorpusId(5),
            threshold: INTENT_FIRE_THRESHOLD,
            policy: PermissionPolicy::deny_all(),
            registry: EffectorRegistry::new(),
            audit_capacity: 1024,
            cooldown_ticks: 5,
            feedback: true,
        }
    }
}

impl AgentBuilder {
    pub fn new() -> Self { Self::default() }
    pub fn motor_corpus(mut self, id: CorpusId) -> Self { self.motor_corpus = id; self }
    pub fn intent_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }
    pub fn policy(mut self, policy: PermissionPolicy) -> Self { self.policy = policy; self }
    pub fn audit_capacity(mut self, capacity: usize) -> Self {
        self.audit_capacity = capacity.max(16);
        self
    }
    pub fn cooldown_ticks(mut self, ticks: u64) -> Self { self.cooldown_ticks = ticks; self }
    pub fn feedback(mut self, enabled: bool) -> Self { self.feedback = enabled; self }

    pub fn with_effector<E: Effector + 'static>(mut self, effector: E) -> Self {
        self.registry.register(effector);
        self
    }

    pub fn build(self) -> AgentResult<AgentLoop> {
        if self.registry.is_empty() {
            return Err(AgentError::Config("no effectors registered; agent would be inert".into()));
        }
        let decoder = IntentDecoder::new(self.motor_corpus, self.threshold);
        Ok(AgentLoop::new(
            decoder,
            self.registry,
            self.policy,
            AuditLog::new(self.audit_capacity),
            self.cooldown_ticks,
            self.feedback,
        ))
    }
}
