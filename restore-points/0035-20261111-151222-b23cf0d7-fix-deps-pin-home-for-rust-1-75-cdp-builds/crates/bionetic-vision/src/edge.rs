//! Sobel edge detection that emits spatially-addressed vision pulses.

use bionetic_core::ids::DotId;
use bionetic_core::multimodal::Modality;
use bionetic_core::pulse::Pulse;

/// Detect edges in a normalized grayscale buffer (f32 in [0.0, 1.0]).
/// Returns pulses only where gradient magnitude exceeds the threshold.
pub fn detect_edges(gray: &[f32], width: usize, height: usize, tick: u64, threshold: f32) -> Vec<Pulse> {
    let expected = match width.checked_mul(height) {
        Some(size) => size,
        None => return Vec::new(),
    };
    if width < 3 || height < 3 || gray.len() != expected {
        return Vec::new();
    }

    let base = Modality::Vision.sensory_offset();
    let capacity = Modality::Vision.capacity().max(1);
    let mut pulses = Vec::new();

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let index = y * width + x;
            let top = (y - 1) * width;
            let middle = y * width;
            let bottom = (y + 1) * width;
            let gx = -gray[top + x - 1] + gray[top + x + 1]
                - 2.0 * gray[middle + x - 1] + 2.0 * gray[middle + x + 1]
                - gray[bottom + x - 1] + gray[bottom + x + 1];
            let gy = -gray[top + x - 1] - 2.0 * gray[top + x] - gray[top + x + 1]
                + gray[bottom + x - 1] + 2.0 * gray[bottom + x] + gray[bottom + x + 1];
            let magnitude = (gx * gx + gy * gy).sqrt().min(1.0);
            if !magnitude.is_finite() || magnitude <= threshold {
                continue;
            }
            let dot_id = DotId(base + (index as u64 % capacity));
            let mut pulse = Pulse::value_pulse(dot_id, magnitude as f64, magnitude as f64, tick);
            pulse.rate.spatial_x = x as f32 / width as f32;
            pulse.rate.spatial_y = y as f32 / height as f32;
            pulses.push(pulse);
        }
    }
    pulses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_edges_on_gradient() {
        let gray: Vec<f32> = (0..25).map(|index| (index % 5) as f32 / 4.0).collect();
        assert!(!detect_edges(&gray, 5, 5, 0, 0.1).is_empty());
    }

    #[test]
    fn test_no_edges_on_uniform() {
        assert!(detect_edges(&vec![0.5; 25], 5, 5, 0, 0.1).is_empty());
    }

    #[test]
    fn test_too_small_frame() {
        assert!(detect_edges(&vec![0.5; 4], 2, 2, 0, 0.0).is_empty());
    }
}
