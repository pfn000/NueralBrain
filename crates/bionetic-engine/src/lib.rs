//! # Bionetic Engine
//!
//! The processing heartbeat of the bionetic brain.
//! Manages tick cycles, pulse delivery, clump management,
//! plasticity, memory consolidation, and sleep states.

pub mod engine;
pub mod processor;
pub mod plasticity;
pub mod memory;
pub mod hippocampus;
pub mod reward;
pub mod dream;
pub mod sleep;

pub use engine::BioneticEngine;
pub mod brain_protocol;
