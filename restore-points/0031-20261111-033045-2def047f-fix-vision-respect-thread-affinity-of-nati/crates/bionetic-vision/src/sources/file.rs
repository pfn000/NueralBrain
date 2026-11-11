//! FileSource — read image files as deterministic frames.

use crate::error::{VisionError, VisionResult};
use crate::frame::{Frame, PixelFormat};
use crate::source::VisualSource;
use std::path::PathBuf;

/// Reads one image or an ordered sequence of image files.
pub struct FileSource {
    paths: Vec<PathBuf>,
    index: usize,
    looping: bool,
    width: u32,
    height: u32,
    name: String,
}

impl FileSource {
    /// Load a single image as a one-shot source.
    pub fn single(path: impl Into<PathBuf>) -> VisionResult<Self> {
        let path = path.into();
        let image = image::open(&path)
            .map_err(|error| VisionError::SourceInit(format!("image open: {error}")))?;
        let (width, height) = (image.width(), image.height());
        Ok(Self {
            paths: vec![path.clone()],
            index: 0,
            looping: false,
            width,
            height,
            name: format!("file:{}", path.display()),
        })
    }

    /// Load an ordered image sequence.
    pub fn sequence(paths: Vec<PathBuf>, looping: bool) -> VisionResult<Self> {
        if paths.is_empty() {
            return Err(VisionError::NoSource);
        }
        let first = image::open(&paths[0])
            .map_err(|error| VisionError::SourceInit(format!("image open: {error}")))?;
        let (width, height) = (first.width(), first.height());
        Ok(Self {
            name: format!("file-seq:{} frames", paths.len()),
            paths,
            index: 0,
            looping,
            width,
            height,
        })
    }
}

impl VisualSource for FileSource {
    fn capture_frame(&mut self) -> VisionResult<Frame> {
        if self.index >= self.paths.len() {
            if self.looping {
                self.index = 0;
            } else {
                return Err(VisionError::CaptureFailed("end of sequence".into()));
            }
        }
        let path = &self.paths[self.index];
        let image = image::open(path)
            .map_err(|error| VisionError::CaptureFailed(format!("image open: {error}")))?;
        let rgb = image.to_rgb8();
        let (width, height) = (rgb.width(), rgb.height());
        let pixels = rgb.into_raw();
        self.index += 1;
        Ok(Frame::new(width, height, PixelFormat::Rgb8, pixels))
    }

    fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
    fn name(&self) -> &str { &self.name }
}
