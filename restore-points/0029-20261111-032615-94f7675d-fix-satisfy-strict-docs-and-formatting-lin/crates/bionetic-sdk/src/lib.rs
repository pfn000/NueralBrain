//! # Bionetic SDK
//!
//! A clean, high-level API for embedding a NeuralBrain into any Rust
//! application. The SDK wraps the lower-level `bionetic-core` and
//! `bionetic-engine` crates behind a builder-style API.
//!
//! ## Example
//!
//! ```no_run
//! use bionetic_sdk::BrainBuilder;
//!
//! let mut brain = BrainBuilder::new()
//!     .with_corpora(8)
//!     .with_dots_per_corpus(128)
//!     .with_dreaming(true)
//!     .build();
//!
//! let output = brain.think(&[0.5, 0.8, 0.1]);
//! brain.learn(&[0.5, 0.8, 0.1], 0.9);
//! brain.dream(100);
//! ```

#![warn(missing_docs)]

pub mod builder;
pub mod handle;
pub mod input;
pub mod output;
pub mod persistence;
pub mod error;

// ── Re-exports: the public API ──
pub use builder::{BrainBuilder, BrainConfig};
pub use handle::BrainHandle;
pub use input::InputBatch;
pub use output::BrainOutput;
pub use persistence::{SaveFormat, StateBundle};
pub use error::{SdkError, SdkResult};

// Re-export commonly-used core types so consumers don't need to depend
// on bionetic-core directly.
pub use bionetic_core::ids::CorpusId;
pub use bionetic_core::chemicals::ChemicalSnapshot;

#[cfg(feature = "multimodal")]
pub use bionetic_core::multimodal::Modality;