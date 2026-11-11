//! Split RGB frames into three parallel pulse channels.

use bionetic_core::ids::DotId;
use bionetic_core::multimodal::Modality;
use bionetic_core::pulse::Pulse;

/// Separate R, G, B channels into three pulse streams occupying the vision region.
pub fn separate_channels(rgb: &[u8], width: usize, tick: u64, noise_floor: u8) -> Vec<Pulse> {
    if width == 0 || rgb.len() % 3 != 0 {
        return Vec::new();
    }
    let pixel_count = rgb.len() / 3;
    let height = if pixel_count % width == 0 {
        pixel_count / width
    } else {
        pixel_count / width + 1
    };
    let base = Modality::Vision.sensory_offset();
    let channel_size = (Modality::Vision.capacity() / 3).max(1);
    let mut pulses = Vec::with_capacity(rgb.len() / 2);

    for (pixel_index, pixel) in rgb.chunks_exact(3).enumerate() {
        let x = pixel_index % width;
        let y = pixel_index / width;
        let offset = pixel_index as u64 % channel_size;

        for (channel, &value) in pixel.iter().enumerate() {
            if value <= noise_floor {
                continue;
            }
            let dot_id = DotId(base + channel as u64 * channel_size + offset);
            let amplitude = value as f32 / 255.0;
            let mut pulse = Pulse::value_pulse(dot_id, amplitude as f64, amplitude as f64, tick);
            pulse.rate.channel = channel as u8;
            pulse.rate.spatial_x = x as f32 / width as f32;
            pulse.rate.spatial_y = y as f32 / height.max(1) as f32;
            pulses.push(pulse);
        }
    }
    pulses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separate_channels() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let pulses = separate_channels(&rgb, 3, 0, 10);
        assert_eq!(pulses.len(), 3);
        assert!(pulses.iter().all(|pulse| pulse.intensity > 0.9));
    }

    #[test]
    fn test_noise_floor_filters() {
        assert!(separate_channels(&[5, 5, 5], 1, 0, 10).is_empty());
    }
}
