use thiserror::Error;

pub type AgentResult<T> = Result<T, AgentError>;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("permission denied for {capability}: {reason}")]
    PermissionDenied { capability: String, reason: String },

    #[error("no effector registered for intent kind {0:?}")]
    NoEffector(crate::intent::IntentKind),

    #[error("effector {0} failed: {1}")]
    EffectorFailed(String, String),

    #[error("effector {0} timed out after {1}ms")]
    Timeout(String, u64),

    #[error("intent decoding failed: {0}")]
    Decode(String),

    #[error("agent loop already running")]
    AlreadyRunning,

    #[error("agent loop not running")]
    NotRunning,

    #[error("invalid configuration: {0}")]
    Config(String),
}
