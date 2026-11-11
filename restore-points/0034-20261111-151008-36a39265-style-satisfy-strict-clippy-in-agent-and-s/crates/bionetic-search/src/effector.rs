#![cfg(feature = "agent")]

use std::sync::Arc;

use parking_lot::Mutex;
use bionetic_agent::effector::{ActionOutcome, Effector, EffectorId};
use bionetic_agent::error::{AgentError, AgentResult};
use bionetic_agent::intent::{Intent, IntentKind};
use bionetic_agent::policy::Capability;

use crate::concept::ConceptId;
use crate::engine::SearchEngine;
use crate::query::QueryBuilder;

/// Maps the first two activation bytes to pre-registered concept slots.
/// Payload byte 1 uses zero for "no secondary concept", otherwise slot = value - 1.
pub struct SearchEffector {
    id: EffectorId,
    engine: Arc<Mutex<SearchEngine>>,
    slots: Vec<ConceptId>,
    limit: usize,
}

impl SearchEffector {
    pub fn new(engine: Arc<Mutex<SearchEngine>>, slots: Vec<ConceptId>) -> Self {
        Self { id: EffectorId::new("search"), engine, slots, limit: crate::DEFAULT_TOP_K }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self
    }
}

impl Effector for SearchEffector {
    fn id(&self) -> EffectorId { self.id.clone() }
    fn handles(&self, intent: &Intent) -> bool { matches!(intent.kind, IntentKind::Query) }

    fn required_capabilities(&self, intent: &Intent) -> Vec<Capability> {
        let wants_web = intent.payload.get(2).copied().unwrap_or(0) & 1 != 0;
        if wants_web { vec![Capability::NetworkHttp] } else { Vec::new() }
    }

    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        if self.slots.is_empty() {
            return Err(AgentError::EffectorFailed(self.id.to_string(), "no concept slots configured".into()));
        }

        let primary = intent.payload.first().copied().unwrap_or(0) as usize;
        let secondary = intent.payload.get(1).copied().unwrap_or(0) as usize;
        let wants_web = intent.payload.get(2).copied().unwrap_or(0) & 1 != 0;

        let mut builder = QueryBuilder::new()
            .concept(self.slots[primary % self.slots.len()])
            .limit(self.limit)
            .web(wants_web);
        if secondary > 0 {
            builder = builder.concept(self.slots[(secondary - 1) % self.slots.len()]);
        }

        let started = std::time::Instant::now();
        let result = self.engine.lock().search(&builder.build()).map_err(|error| {
            AgentError::EffectorFailed(self.id.to_string(), error.to_string())
        })?;

        let mut output = format!(
            "{} hits / {} local candidates / {}us\n",
            result.hits.len(), result.total_candidates, result.latency_us
        ).into_bytes();
        for hit in &result.hits {
            output.extend_from_slice(hit.title.as_bytes());
            output.push(b'\n');
            output.extend_from_slice(hit.snippet.as_bytes());
            output.push(b'\n');
        }

        Ok(ActionOutcome::success(output, started.elapsed().as_micros() as u64))
    }
}
