//! Frame representation — the common currency between sources and the pipeline.

use serde::{Deserialize, Serialize};

/// Pixel layout of a captured frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// 8-bit grayscale, one byte per pixel.
    Grayscale8,
    /// 24-bit RGB, three bytes per pixel, row-major.
    Rgb8,
    /// 32-bit RGBA, four bytes per pixel, row-major.
    Rgba8,
}

impl PixelFormat {
    /// Number of bytes used by one pixel in this format.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Grayscale8 => 1,
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgba8 => 4,
        }
    }
}

/// A single captured image frame.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Pixel byte layout.
    pub format: PixelFormat,
    /// Row-major pixel bytes.
    pub pixels: Vec<u8>,
    /// Source timestamp (monotonic tick, milliseconds since epoch, or zero).
    pub timestamp: u64,
}

impl Frame {
    /// Create a frame with timestamp zero.
    pub fn new(width: u32, height: u32, format: PixelFormat, pixels: Vec<u8>) -> Self {
        Self { width, height, format, pixels, timestamp: 0 }
    }

    /// Total declared pixel count, saturating on narrow address spaces.
    pub fn pixel_count(&self) -> usize {
        (self.width as usize).saturating_mul(self.height as usize)
    }

    /// Verify that pixel length matches dimensions and pixel format.
    pub fn is_valid(&self) -> bool {
        self.pixel_count()
            .checked_mul(self.format.bytes_per_pixel())
            .map(|expected| expected == self.pixels.len())
            .unwrap_or(false)
    }

    /// Convert this frame to grayscale (owned copy). Grayscale frames are returned as-is.
    pub fn to_grayscale(&self) -> Frame {
        match self.format {
            PixelFormat::Grayscale8 => self.clone(),
            PixelFormat::Rgb8 => {
                let gray: Vec<u8> = self.pixels.chunks_exact(3).map(|pixel| {
                    (pixel[0] as f32 * 0.299 + pixel[1] as f32 * 0.587 + pixel[2] as f32 * 0.114) as u8
                }).collect();
                Frame { width: self.width, height: self.height, format: PixelFormat::Grayscale8, pixels: gray, timestamp: self.timestamp }
            }
            PixelFormat::Rgba8 => {
                let gray: Vec<u8> = self.pixels.chunks_exact(4).map(|pixel| {
                    (pixel[0] as f32 * 0.299 + pixel[1] as f32 * 0.587 + pixel[2] as f32 * 0.114) as u8
                }).collect();
                Frame { width: self.width, height: self.height, format: PixelFormat::Grayscale8, pixels: gray, timestamp: self.timestamp }
            }
        }
    }

    /// Normalize grayscale pixel intensities to f32 in the [0, 1] interval.
    pub fn to_grayscale_f32(&self) -> Vec<f32> {
        self.to_grayscale().pixels.iter().map(|&pixel| pixel as f32 / 255.0).collect()
    }

    /// Return RGB bytes, expanding grayscale or dropping alpha as needed.
    pub fn to_rgb(&self) -> Vec<u8> {
        match self.format {
            PixelFormat::Rgb8 => self.pixels.clone(),
            PixelFormat::Grayscale8 => {
                let mut rgb = Vec::with_capacity(self.pixels.len().saturating_mul(3));
                for &gray in &self.pixels {
                    rgb.extend_from_slice(&[gray, gray, gray]);
                }
                rgb
            }
            PixelFormat::Rgba8 => self.pixels.chunks_exact(4)
                .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]])
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_validity() {
        let frame = Frame::new(10, 10, PixelFormat::Grayscale8, vec![0u8; 100]);
        assert!(frame.is_valid());
        let invalid = Frame::new(10, 10, PixelFormat::Rgb8, vec![0u8; 100]);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_rgb_to_grayscale() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let gray = Frame::new(3, 1, PixelFormat::Rgb8, rgb).to_grayscale();
        assert_eq!(gray.format, PixelFormat::Grayscale8);
        assert_eq!(gray.pixels.len(), 3);
    }

    #[test]
    fn test_grayscale_to_rgb() {
        let frame = Frame::new(2, 1, PixelFormat::Grayscale8, vec![128, 64]);
        assert_eq!(frame.to_rgb(), vec![128, 128, 128, 64, 64, 64]);
    }

    #[test]
    fn test_grayscale_f32() {
        let frame = Frame::new(2, 1, PixelFormat::Grayscale8, vec![255, 0]);
        assert_eq!(frame.to_grayscale_f32(), vec![1.0, 0.0]);
    }
}
