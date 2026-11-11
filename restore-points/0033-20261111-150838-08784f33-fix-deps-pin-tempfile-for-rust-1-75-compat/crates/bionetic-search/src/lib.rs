//! Brain-driven retrieval.
//!
//! Concepts are stable symbolic anchors selected through motor slots. Text is
//! tokenised only in this retrieval layer, never in the dot layer. Results can
//! be encoded as sensory pulse patterns so repeated exposure can shape
//! concept-to-content associations without an LLM or token vocabulary in the brain.

pub mod concept;
pub mod corpus;
pub mod encoder;
pub mod engine;
pub mod error;
pub mod index;
pub mod query;
pub mod ranker;
pub mod result;

#[cfg(feature = "web")]
pub mod web;
#[cfg(feature = "agent")]
pub mod effector;

pub use concept::{Concept, ConceptId};
pub use corpus::{CustomCorpus, Document, DocumentId};
pub use encoder::{SearchEncoder, SearchEncoderConfig};
pub use engine::{SearchEngine, SearchEngineConfig};
pub use error::{SearchError, SearchResult as SearchErrResult};
pub use index::InvertedIndex;
pub use query::{Query, QueryBuilder};
pub use ranker::{score_bm25lite, RankerConfig};
pub use result::{Hit, SearchResult};

pub const DEFAULT_TOP_K: usize = 8;
pub const DOTS_PER_HIT: usize = 16;
