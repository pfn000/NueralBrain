//! Configurable vision pipeline orchestrating edges, color, motion, and context binding.

use crate::color;
use crate::context::ContextBundle;
use crate::edge;
use crate::frame::Frame;
use crate::motion;
use bionetic_core::pulse::Pulse;
use serde::{Deserialize, Serialize};

/// Configuration for the visual perception stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionPipelineConfig {
    /// Enable Sobel edge pulses.
    pub enable_edges: bool,
    /// Enable RGB channel pulses.
    pub enable_color: bool,
    /// Enable inter-frame motion pulses.
    pub enable_motion: bool,
    /// Bind stage pulses into a context clump before flattening.
    pub enable_context_binding: bool,
    /// Minimum Sobel magnitude emitted as an edge pulse.
    pub edge_threshold: f32,
    /// Minimum inter-frame intensity delta emitted as motion.
    pub motion_threshold: f32,
    /// Ignore RGB channel values at or below this noise floor.
    pub color_noise_floor: u8,
}

impl Default for VisionPipelineConfig {
    fn default() -> Self {
        Self {
            enable_edges: true,
            enable_color: true,
            enable_motion: true,
            enable_context_binding: true,
            edge_threshold: 0.15,
            motion_threshold: 0.05,
            color_noise_floor: 10,
        }
    }
}

/// Stateful pipeline that retains the previous grayscale frame for motion detection.
pub struct VisionPipeline {
    config: VisionPipelineConfig,
    prev_gray: Option<Vec<f32>>,
    prev_dims: (usize, usize),
}

impl VisionPipeline {
    /// Create a pipeline with the supplied configuration.
    pub fn new(config: VisionPipelineConfig) -> Self {
        Self { config, prev_gray: None, prev_dims: (0, 0) }
    }

    /// Process a frame and return pulse objects ready for SDK or engine injection.
    pub fn process_frame(&mut self, frame: &Frame, tick: u64) -> Vec<Pulse> {
        let (width, height) = (frame.width as usize, frame.height as usize);
        if width == 0 || height == 0 || !frame.is_valid() {
            self.reset();
            return Vec::new();
        }
        let gray = frame.to_grayscale_f32();
        let rgb = frame.to_rgb();
        let edge_pulses = if self.config.enable_edges {
            edge::detect_edges(&gray, width, height, tick, self.config.edge_threshold)
        } else { Vec::new() };
        let color_pulses = if self.config.enable_color {
            color::separate_channels(&rgb, width, tick, self.config.color_noise_floor)
        } else { Vec::new() };
        let motion_pulses = if self.config.enable_motion {
            match &self.prev_gray {
                Some(previous) if self.prev_dims == (width, height) =>
                    motion::detect_motion(previous, &gray, width, tick, self.config.motion_threshold),
                _ => Vec::new(),
            }
        } else { Vec::new() };

        self.prev_gray = Some(gray);
        self.prev_dims = (width, height);

        if self.config.enable_context_binding {
            ContextBundle::bind(edge_pulses, color_pulses, motion_pulses, tick).into_pulses()
        } else {
            let mut pulses = edge_pulses;
            pulses.extend(color_pulses);
            pulses.extend(motion_pulses);
            pulses
        }
    }

    /// Read the current configuration.
    pub fn config(&self) -> &VisionPipelineConfig { &self.config }
    /// Mutably access the current configuration.
    pub fn config_mut(&mut self) -> &mut VisionPipelineConfig { &mut self.config }

    /// Reset the stored previous frame, for example after a scene cut.
    pub fn reset(&mut self) {
        self.prev_gray = None;
        self.prev_dims = (0, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PixelFormat;

    fn sample_frame(seed: u8) -> Frame {
        let pixels: Vec<u8> = (0..300)
            .map(|index| (index as u8).wrapping_add(seed) % 255)
            .collect();
        Frame::new(10, 10, PixelFormat::Rgb8, pixels)
    }

    #[test]
    fn test_pipeline_produces_pulses() {
        let mut pipeline = VisionPipeline::new(VisionPipelineConfig::default());
        assert!(!pipeline.process_frame(&sample_frame(0), 1).is_empty());
    }

    #[test]
    fn test_motion_appears_on_second_frame() {
        let mut pipeline = VisionPipeline::new(VisionPipelineConfig::default());
        let _ = pipeline.process_frame(&sample_frame(0), 1);
        assert!(!pipeline.process_frame(&sample_frame(100), 2).is_empty());
    }

    #[test]
    fn test_disabling_stages() {
        let config = VisionPipelineConfig {
            enable_edges: false,
            enable_color: false,
            enable_motion: false,
            ..VisionPipelineConfig::default()
        };
        let mut pipeline = VisionPipeline::new(config);
        assert!(pipeline.process_frame(&sample_frame(0), 1).is_empty());
    }

    #[test]
    fn test_reset_clears_motion_state() {
        let mut pipeline = VisionPipeline::new(VisionPipelineConfig::default());
        let _ = pipeline.process_frame(&sample_frame(0), 1);
        pipeline.reset();
        let pulses = pipeline.process_frame(&sample_frame(100), 2);
        assert!(pulses.iter().all(|pulse| !matches!(
            pulse.pulse_type,
            bionetic_core::pulse::PulseType::Modulatory
        )));
    }

    #[test]
    fn test_invalid_frame_is_rejected_without_panicking() {
        let mut pipeline = VisionPipeline::new(VisionPipelineConfig::default());
        let invalid = Frame::new(10, 10, PixelFormat::Rgb8, vec![0; 10]);
        assert!(pipeline.process_frame(&invalid, 1).is_empty());
    }
}
