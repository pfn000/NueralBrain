//! Temporal motion detection via frame differencing.

use bionetic_core::ids::DotId;
use bionetic_core::multimodal::Modality;
use bionetic_core::pulse::{Pulse, PulseType};

/// Compare consecutive grayscale frames and emit pulses where the delta exceeds the threshold.
pub fn detect_motion(prev: &[f32], curr: &[f32], width: usize, tick: u64, threshold: f32) -> Vec<Pulse> {
    if width == 0 || prev.len() != curr.len() {
        return Vec::new();
    }
    let height = if prev.len() % width == 0 {
        prev.len() / width
    } else {
        prev.len() / width + 1
    };
    let base = Modality::Vision.sensory_offset();
    let capacity = Modality::Vision.capacity().max(1);

    prev.iter().zip(curr.iter()).enumerate().filter_map(|(index, (&before, &after))| {
        if !before.is_finite() || !after.is_finite() {
            return None;
        }
        let delta = (after - before).abs();
        if delta <= threshold {
            return None;
        }
        let x = index % width;
        let y = index / width;
        let dot_id = DotId(base + (index as u64 % capacity));
        let amplitude = delta.min(1.0);
        let mut pulse = Pulse::value_pulse(dot_id, amplitude as f64, amplitude as f64, tick);
        pulse.pulse_type = PulseType::Modulatory;
        pulse.rate.spatial_x = x as f32 / width as f32;
        pulse.rate.spatial_y = y as f32 / height.max(1) as f32;
        pulse.rate.temporal_phase = amplitude;
        Some(pulse)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motion_detected_on_change() {
        assert_eq!(detect_motion(&vec![0.0; 9], &vec![1.0; 9], 3, 0, 0.1).len(), 9);
    }

    #[test]
    fn test_no_motion_on_identical() {
        let frame = vec![0.5; 9];
        assert!(detect_motion(&frame, &frame, 3, 0, 0.01).is_empty());
    }

    #[test]
    fn test_mismatched_sizes_return_empty() {
        assert!(detect_motion(&[0.0; 4], &[0.0; 9], 3, 0, 0.0).is_empty());
    }
}
