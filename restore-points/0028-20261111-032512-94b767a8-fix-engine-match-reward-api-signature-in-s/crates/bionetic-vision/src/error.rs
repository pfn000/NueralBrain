//! Vision-specific errors.

use thiserror::Error;

/// Result alias used by the vision crate.
pub type VisionResult<T> = Result<T, VisionError>;

/// Errors returned by frame sources and vision stages.
#[derive(Debug, Error)]
pub enum VisionError {
    /// No paths or capture devices were available.
    #[error("no source available")]
    NoSource,
    /// A camera, browser adapter, or file source could not be initialized.
    #[error("source initialization failed: {0}")]
    SourceInit(String),
    /// A source failed while producing the next frame.
    #[error("frame capture failed: {0}")]
    CaptureFailed(String),
    /// A frame declared zero or invalid dimensions.
    #[error("invalid frame dimensions: {w}x{h}")]
    InvalidDimensions { w: u32, h: u32 },
    /// A source returned a pixel format that its adapter cannot decode.
    #[error("unsupported pixel format: {0}")]
    UnsupportedFormat(String),
    /// A local filesystem operation failed.
    #[error("io error: {0}")]
    Io(String),
    /// An underlying camera or browser backend returned an error.
    #[error("backend error: {0}")]
    Backend(String),
}

impl From<std::io::Error> for VisionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
