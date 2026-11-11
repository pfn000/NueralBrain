//! # Bionetic Core
//!
//! Core primitives for the Bionetic Processing Architecture.
//! Defines Dots, Pulses, Gates, Rates, and all fundamental types.

pub mod ids;
pub mod chemicals;
pub mod rate;
pub mod gate;
pub mod dot;
pub mod pulse;
pub mod corpus;
pub mod clump;
pub mod bundle;
pub mod trans;
pub mod neurogenesis;
pub mod multimodal;
pub mod region;

// Re-exports
pub use ids::*;
pub use chemicals::*;
pub use rate::*;
pub use gate::*;
pub use dot::*;
pub use pulse::*;
pub use corpus::*;
pub use clump::*;
pub use bundle::*;
pub use trans::*;
pub use region::{RegionState, RegionStatus};
