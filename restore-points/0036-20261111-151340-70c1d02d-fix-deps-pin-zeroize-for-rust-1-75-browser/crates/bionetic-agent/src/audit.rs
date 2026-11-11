use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;

use crate::effector::EffectorId;
use crate::intent::{Intent, IntentKind};
use crate::policy::Capability;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AuditOutcome {
    Executed { ok: bool, latency_us: u64, bytes: usize },
    Blocked { capability: String, reason: String },
    NoEffector,
    DecodedIdle,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuditEntry {
    pub ts_ms: u64,
    pub tick: u64,
    pub intent_kind: IntentKind,
    pub intent_strength: f32,
    pub effector_id: Option<String>,
    pub capabilities: Vec<String>,
    pub outcome: AuditOutcome,
}

#[derive(Clone)]
pub struct AuditLog {
    inner: Arc<Mutex<VecDeque<AuditEntry>>>,
    capacity: usize,
}

impl AuditLog {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self { inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))), capacity }
    }

    pub fn push(
        &self,
        intent: &Intent,
        effector_id: Option<&EffectorId>,
        capabilities: &[Capability],
        outcome: AuditOutcome,
    ) {
        let entry = AuditEntry {
            ts_ms: now_ms(),
            tick: intent.tick,
            intent_kind: intent.kind,
            intent_strength: intent.strength,
            effector_id: effector_id.map(ToString::to_string),
            capabilities: capabilities.iter().map(Capability::as_label).collect(),
            outcome,
        };
        let mut entries = self.inner.lock();
        if entries.len() >= self.capacity { entries.pop_front(); }
        entries.push_back(entry);
    }

    pub fn snapshot(&self) -> Vec<AuditEntry> {
        self.inner.lock().iter().cloned().collect()
    }

    pub fn len(&self) -> usize { self.inner.lock().len() }
    pub fn is_empty(&self) -> bool { self.inner.lock().is_empty() }
    pub fn clear(&self) { self.inner.lock().clear(); }
}

impl Default for AuditLog {
    fn default() -> Self { Self::new(1024) }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
