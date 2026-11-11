//! CameraSource — webcam capture via nokhwa.
//!
//! Native backends map to V4L2 on Linux, MSMF on Windows, and AVFoundation on macOS.

use crate::error::{VisionError, VisionResult};
use crate::frame::{Frame, PixelFormat};
use crate::source::VisualSource;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution},
    Camera,
};

/// Captures frames from a system camera (webcam or USB camera).
pub struct CameraSource {
    camera: Camera,
    width: u32,
    height: u32,
    name: String,
}

impl CameraSource {
    /// Open the default camera at the requested resolution.
    pub fn open_default(width: u32, height: u32) -> VisionResult<Self> {
        Self::open_index(0, width, height)
    }

    /// Open a specific camera by index.
    pub fn open_index(index: u32, width: u32, height: u32) -> VisionResult<Self> {
        if width == 0 || height == 0 {
            return Err(VisionError::InvalidDimensions { w: width, h: height });
        }
        let requested = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::Closest(CameraFormat::new(
                Resolution::new(width, height),
                FrameFormat::RAWRGB,
                30,
            )),
        );
        let mut camera = Camera::new(CameraIndex::Index(index), requested)
            .map_err(|error| VisionError::SourceInit(format!("nokhwa open: {error}")))?;
        camera.open_stream()
            .map_err(|error| VisionError::SourceInit(format!("open_stream: {error}")))?;
        let resolution = camera.resolution();
        let name = format!("camera[{index}]@{}x{}", resolution.width_x, resolution.height_y);
        Ok(Self {
            camera,
            width: resolution.width_x,
            height: resolution.height_y,
            name,
        })
    }

    /// Enumerate available cameras without opening them.
    pub fn enumerate() -> VisionResult<Vec<String>> {
        let devices = nokhwa::query(ApiBackend::Auto)
            .map_err(|error| VisionError::Backend(format!("enumerate: {error}")))?;
        Ok(devices.into_iter().enumerate()
            .map(|(index, device)| format!("[{index}] {}", device.human_name()))
            .collect())
    }
}

impl VisualSource for CameraSource {
    fn capture_frame(&mut self) -> VisionResult<Frame> {
        let buffer = self.camera.frame()
            .map_err(|error| VisionError::CaptureFailed(format!("nokhwa frame: {error}")))?;
        let decoded = buffer.decode_image::<RgbFormat>()
            .map_err(|error| VisionError::CaptureFailed(format!("decode frame: {error}")))?;
        let width = decoded.width();
        let height = decoded.height();
        let pixels = decoded.into_raw();
        let mut frame = Frame::new(width, height, PixelFormat::Rgb8, pixels);
        frame.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);
        Ok(frame)
    }

    fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
    fn name(&self) -> &str { &self.name }

    fn close(&mut self) -> VisionResult<()> {
        self.camera.stop_stream()
            .map_err(|error| VisionError::Backend(format!("stop_stream: {error}")))?;
        Ok(())
    }
}
