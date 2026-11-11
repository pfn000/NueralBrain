//! Agent core: a permission-gated bridge between motor activity and effectors.
//!
//! The engine remains independent of network, filesystem, shell, browser, and
//! code-emission concerns. Every concrete action is selected by an effector
//! and checked against an explicit policy before execution.

pub mod audit;
pub mod builder;
pub mod effector;
pub mod error;
pub mod intent;
pub mod loop_;
pub mod noop;
pub mod policy;
pub mod registry;
pub mod effectors;

pub use audit::{AuditEntry, AuditLog, AuditOutcome};
pub use builder::AgentBuilder;
pub use effector::{ActionOutcome, Effector, EffectorId};
pub use error::{AgentError, AgentResult};
pub use intent::{Intent, IntentDecoder, IntentKind};
pub use loop_::{AgentLoop, AgentTickReport};
pub use noop::NoopEffector;
pub use policy::{Capability, Decision, PermissionPolicy};
pub use registry::EffectorRegistry;

/// Number of engine ticks considered when decoding recent motor activity.
pub const INTENT_WINDOW_TICKS: u64 = 10;

/// Fraction of active dots in a motor region required to decode an intent.
pub const INTENT_FIRE_THRESHOLD: f32 = 0.35;
