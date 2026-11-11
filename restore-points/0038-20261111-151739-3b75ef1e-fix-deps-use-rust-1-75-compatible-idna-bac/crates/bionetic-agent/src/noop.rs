use crate::effector::{ActionOutcome, Effector, EffectorId};
use crate::error::AgentResult;
use crate::intent::{Intent, IntentKind};
use crate::policy::Capability;

pub struct NoopEffector {
    id: EffectorId,
    accepts: Option<IntentKind>,
}

impl NoopEffector {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: EffectorId::new(id), accepts: None }
    }

    pub fn only(mut self, kind: IntentKind) -> Self {
        self.accepts = Some(kind);
        self
    }
}

impl Effector for NoopEffector {
    fn id(&self) -> EffectorId { self.id.clone() }

    fn handles(&self, intent: &Intent) -> bool {
        match self.accepts {
            Some(kind) => kind == intent.kind,
            None => !matches!(intent.kind, IntentKind::Idle),
        }
    }

    fn required_capabilities(&self, _intent: &Intent) -> Vec<Capability> { Vec::new() }

    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        Ok(ActionOutcome::success(
            format!("noop handled {:?}", intent.kind).into_bytes(),
            0,
        ))
    }
}
