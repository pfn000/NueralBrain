use std::time::Instant;

use bionetic_core::ids::CorpusId;
use bionetic_engine::engine::BioneticEngine;

use crate::audit::{AuditLog, AuditOutcome};
use crate::effector::ActionOutcome;
use crate::error::{AgentError, AgentResult};
use crate::intent::{Intent, IntentDecoder, IntentKind};
use crate::policy::{Decision, PermissionPolicy};
use crate::registry::EffectorRegistry;

#[derive(Debug, Clone)]
pub struct AgentTickReport {
    pub intent: Intent,
    pub executed: bool,
    pub blocked: bool,
    pub outcome: Option<ActionOutcome>,
    pub effector_id: Option<String>,
}

pub struct AgentLoop {
    decoder: IntentDecoder,
    registry: EffectorRegistry,
    policy: PermissionPolicy,
    audit: AuditLog,
    cooldown_ticks: u64,
    last_dispatch_tick: std::collections::HashMap<IntentKind, u64>,
    feedback_enabled: bool,
}

impl AgentLoop {
    pub(crate) fn new(
        decoder: IntentDecoder,
        registry: EffectorRegistry,
        policy: PermissionPolicy,
        audit: AuditLog,
        cooldown_ticks: u64,
        feedback_enabled: bool,
    ) -> Self {
        Self {
            decoder,
            registry,
            policy,
            audit,
            cooldown_ticks,
            last_dispatch_tick: Default::default(),
            feedback_enabled,
        }
    }

    pub fn audit(&self) -> &AuditLog { &self.audit }
    pub fn policy(&self) -> &PermissionPolicy { &self.policy }
    pub fn registry(&self) -> &EffectorRegistry { &self.registry }

    /// Decode the current motor state and optionally dispatch one permitted action.
    pub fn tick(&mut self, engine: &mut BioneticEngine) -> AgentResult<AgentTickReport> {
        let intent = self.decoder.decode(engine);

        if matches!(intent.kind, IntentKind::Idle) {
            self.audit.push(&intent, None, &[], AuditOutcome::DecodedIdle);
            return Ok(AgentTickReport {
                intent, executed: false, blocked: false, outcome: None, effector_id: None,
            });
        }

        if let Some(&last) = self.last_dispatch_tick.get(&intent.kind) {
            if intent.tick.saturating_sub(last) < self.cooldown_ticks {
                return Ok(AgentTickReport {
                    intent, executed: false, blocked: false, outcome: None, effector_id: None,
                });
            }
        }

        let effector = match self.registry.find(&intent) {
            Some(effector) => effector,
            None => {
                self.audit.push(&intent, None, &[], AuditOutcome::NoEffector);
                return Err(AgentError::NoEffector(intent.kind));
            }
        };

        let capabilities = effector.required_capabilities(&intent);
        for capability in &capabilities {
            if self.policy.evaluate(&intent, capability) == Decision::Deny {
                self.audit.push(
                    &intent,
                    Some(&effector.id()),
                    &capabilities,
                    AuditOutcome::Blocked {
                        capability: capability.as_label(),
                        reason: "policy denied".into(),
                    },
                );
                return Ok(AgentTickReport {
                    intent,
                    executed: false,
                    blocked: true,
                    outcome: None,
                    effector_id: Some(effector.id().to_string()),
                });
            }
        }

        let started = Instant::now();
        let result = effector.execute(&intent);
        let elapsed_us = started.elapsed().as_micros() as u64;
        self.last_dispatch_tick.insert(intent.kind, intent.tick);

        match result {
            Ok(outcome) => {
                self.audit.push(
                    &intent,
                    Some(&effector.id()),
                    &capabilities,
                    AuditOutcome::Executed {
                        ok: outcome.ok,
                        latency_us: outcome.latency_us.max(elapsed_us),
                        bytes: outcome.bytes.len(),
                    },
                );
                if self.feedback_enabled && outcome.ok {
                    feed_bytes_to_sensory(engine, &outcome.bytes);
                }
                Ok(AgentTickReport {
                    intent,
                    executed: outcome.ok,
                    blocked: false,
                    effector_id: Some(effector.id().to_string()),
                    outcome: Some(outcome),
                })
            }
            Err(error) => {
                self.audit.push(
                    &intent,
                    Some(&effector.id()),
                    &capabilities,
                    AuditOutcome::Executed { ok: false, latency_us: elapsed_us, bytes: 0 },
                );
                Err(AgentError::EffectorFailed(effector.id().to_string(), error.to_string()))
            }
        }
    }

    /// Advance and process n engine ticks. Recoverable dispatch errors are already audited.
    pub fn run(&mut self, engine: &mut BioneticEngine, n: u64) -> Vec<AgentTickReport> {
        let mut reports = Vec::with_capacity(n.min(usize::MAX as u64) as usize);
        for _ in 0..n {
            engine.tick();
            if let Ok(report) = self.tick(engine) { reports.push(report); }
        }
        reports
    }
}

fn feed_bytes_to_sensory(engine: &mut BioneticEngine, bytes: &[u8]) {
    let sensory = CorpusId(0);
    let Some(corpus) = engine.corpora.get(&sensory) else { return; };
    let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
    ids.sort_unstable();
    ids.truncate(bytes.len());
    for (dot_id, byte) in ids.into_iter().zip(bytes.iter().copied()) {
        let amplitude = byte as f64 / 255.0;
        if amplitude >= 0.05 {
            engine.inject_pulse(dot_id, sensory, amplitude, 0);
        }
    }
}
