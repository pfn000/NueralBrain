//! The VisualSource trait — all frame producers implement this.

use crate::error::VisionResult;
use crate::frame::Frame;

/// Any producer of frames: cameras, browser adapters, file readers, and streams.
/// Implementations are not required to be Send: native camera backends such as
/// nokhwa may contain platform handles that are intentionally thread-bound.
/// Move or synchronize a source only when its concrete backend supports it.
pub trait VisualSource {
    /// Capture one frame (blocking or best-available).
    fn capture_frame(&mut self) -> VisionResult<Frame>;
    /// Current source dimensions, which may change after reconfiguration.
    fn dimensions(&self) -> (u32, u32);
    /// Human-readable source name for logging.
    fn name(&self) -> &str;
    /// Gracefully shut down the source.
    fn close(&mut self) -> VisionResult<()> { Ok(()) }
}
