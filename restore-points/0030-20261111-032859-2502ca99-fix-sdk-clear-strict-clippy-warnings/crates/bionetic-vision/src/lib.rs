//! # bionetic-vision
//!
//! Converts frames from a VisualSource into edge, color, and motion pulse streams
//! for NeuralBrain's sensory cortex.

#![warn(missing_docs)]

pub mod color;
pub mod context;
pub mod edge;
pub mod error;
pub mod frame;
pub mod motion;
pub mod pipeline;
pub mod source;
pub mod sources;

/// Vision-specific error types.
pub use error::{VisionError, VisionResult};
/// Frame representation and pixel format.
pub use frame::{Frame, PixelFormat};
/// Stateful visual processing pipeline.
pub use pipeline::{VisionPipeline, VisionPipelineConfig};
/// Source interface for camera, browser, and file producers.
pub use source::VisualSource;
