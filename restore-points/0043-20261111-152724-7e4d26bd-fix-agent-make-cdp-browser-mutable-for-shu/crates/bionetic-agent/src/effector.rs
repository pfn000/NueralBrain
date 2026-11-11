use crate::error::AgentResult;
use crate::intent::Intent;
use crate::policy::Capability;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectorId(pub String);

impl EffectorId {
    pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
}

impl std::fmt::Display for EffectorId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ActionOutcome {
    pub ok: bool,
    pub bytes: Vec<u8>,
    pub message: String,
    pub latency_us: u64,
}

impl ActionOutcome {
    pub fn success(bytes: Vec<u8>, latency_us: u64) -> Self {
        Self { ok: true, bytes, message: "ok".into(), latency_us }
    }

    pub fn failure(message: impl Into<String>, latency_us: u64) -> Self {
        Self { ok: false, bytes: Vec::new(), message: message.into(), latency_us }
    }
}

/// An effector's capabilities are checked by AgentLoop before execute is called.
pub trait Effector: Send + Sync {
    fn id(&self) -> EffectorId;
    fn handles(&self, intent: &Intent) -> bool;
    fn required_capabilities(&self, intent: &Intent) -> Vec<Capability>;
    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome>;
    fn shutdown(&self) {}
}
