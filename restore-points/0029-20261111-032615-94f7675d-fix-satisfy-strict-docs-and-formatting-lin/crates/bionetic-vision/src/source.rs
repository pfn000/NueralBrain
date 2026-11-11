//! The VisualSource trait — all frame producers implement this.

use crate::error::VisionResult;
use crate::frame::Frame;

/// Any producer of frames: cameras, browser adapters, file readers, and streams.
pub trait VisualSource: Send {
    /// Capture one frame (blocking or best-available).
    fn capture_frame(&mut self) -> VisionResult<Frame>;
    /// Current source dimensions, which may change after reconfiguration.
    fn dimensions(&self) -> (u32, u32);
    /// Human-readable source name for logging.
    fn name(&self) -> &str;
    /// Gracefully shut down the source.
    fn close(&mut self) -> VisionResult<()> { Ok(()) }
}
