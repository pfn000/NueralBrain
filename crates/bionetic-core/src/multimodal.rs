//! # Multi-modal Input
//!
//! Converts raw sensory signals into pulse trains for the sensory corpus.
//! This is deliberately NOT a tokenizer and NOT an embedding model.
//! Each modality maps to a logical region of sensory dot-ID space so
//! cross-modal associations can be learned through the existing plasticity path.
//!
//! The IDs emitted here are logical sensory addresses. BioneticEngine resolves
//! them to physical dots in CorpusId(0), allocating one per address until the
//! configured sensory-corpus capacity is reached.

use crate::dot::DotId;
use crate::pulse::{Pulse, PulseType};
use crate::rate::Rate;
use serde::{Deserialize, Serialize};

/// A sensory modality. Each one owns a logical region of dot-ID space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    Vision,
    Audio,
    Text,
    Touch,
    Proprioceptive,
    Custom(u8),
}

impl Modality {
    /// Base dot-ID offset for this modality within the sensory corpus.
    pub fn sensory_offset(&self) -> u64 {
        match self {
            Modality::Vision => 0,
            Modality::Audio => 1024,
            Modality::Text => 2048,
            Modality::Touch => 3072,
            Modality::Proprioceptive => 3584,
            Modality::Custom(n) => 4096 + (*n as u64 * 256),
        }
    }

    /// Maximum logical dots reserved for this modality.
    pub fn capacity(&self) -> u64 {
        match self {
            Modality::Vision => 1024,
            Modality::Audio => 1024,
            Modality::Text => 1024,
            Modality::Touch => 512,
            Modality::Proprioceptive => 256,
            Modality::Custom(_) => 256,
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Modality::Vision => "vision",
            Modality::Audio => "audio",
            Modality::Text => "text",
            Modality::Touch => "touch",
            Modality::Proprioceptive => "proprioceptive",
            Modality::Custom(_) => "custom",
        }
    }
}

/// Build a pulse while preserving signed signal values and normalized intensity.
#[inline]
fn make_pulse(
    dot_id: u64,
    value: f32,
    amplitude: f32,
    tick: u64,
    pulse_type: PulseType,
    rate: Rate,
) -> Pulse {
    let safe_value = if value.is_finite() { value as f64 } else { 0.0 };
    let safe_amplitude = if amplitude.is_finite() {
        amplitude.clamp(0.0, 1.0) as f64
    } else {
        0.0
    };

    let mut pulse = Pulse::value_pulse(
        DotId(dot_id),
        safe_value,
        safe_amplitude,
        tick,
    );
    pulse.pulse_type = pulse_type;
    pulse.rate = rate;
    pulse
}

/// Convert a grayscale pixel buffer (u8, row-major) to pulses.
/// Each pixel maps to one logical sensory dot; brightness maps to amplitude.
/// Pixels at or below noise_floor are omitted.
pub fn image_grayscale_to_pulses(
    pixels: &[u8],
    width: usize,
    height: usize,
    tick: u64,
    noise_floor: u8,
) -> Vec<Pulse> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let base = Modality::Vision.sensory_offset();
    let cap = Modality::Vision.capacity();
    let pixel_count = pixels.len().min(width.saturating_mul(height));
    let mut pulses = Vec::with_capacity(pixel_count / 4);

    for (index, &brightness) in pixels.iter().take(pixel_count).enumerate() {
        if brightness <= noise_floor {
            continue;
        }

        let amplitude = brightness as f32 / 255.0;
        let mut rate = Rate::default();
        rate.spatial_x = (index % width) as f32 / width as f32;
        rate.spatial_y = (index / width) as f32 / height as f32;

        pulses.push(make_pulse(
            base + (index as u64 % cap),
            amplitude,
            amplitude,
            tick,
            PulseType::Value,
            rate,
        ));
    }

    pulses
}

/// Convert an RGB buffer (3 bytes per pixel, row-major) to parallel
/// red, green, and blue pulse streams within the vision region.
pub fn image_rgb_to_pulses(
    rgb: &[u8],
    width: usize,
    height: usize,
    tick: u64,
    noise_floor: u8,
) -> Vec<Pulse> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let base = Modality::Vision.sensory_offset();
    let channel_size = Modality::Vision.capacity() / 3;
    let pixel_count = (rgb.len() / 3).min(width.saturating_mul(height));
    let mut pulses = Vec::with_capacity(pixel_count.saturating_mul(3));

    for (index, chunk) in rgb.chunks_exact(3).take(pixel_count).enumerate() {
        let offset = index as u64 % channel_size;

        for (channel, &value) in chunk.iter().enumerate() {
            if value <= noise_floor {
                continue;
            }

            let amplitude = value as f32 / 255.0;
            let mut rate = Rate::default();
            rate.spatial_x = (index % width) as f32 / width as f32;
            rate.spatial_y = (index / width) as f32 / height as f32;
            rate.channel = channel as u8;

            pulses.push(make_pulse(
                base + channel as u64 * channel_size + offset,
                amplitude,
                amplitude,
                tick,
                PulseType::Value,
                rate,
            ));
        }
    }

    pulses
}

/// Convert normalized FFT frequency-band amplitudes to pulses.
pub fn audio_bands_to_pulses(bands: &[f32], tick: u64, noise_floor: f32) -> Vec<Pulse> {
    let base = Modality::Audio.sensory_offset();
    let cap = Modality::Audio.capacity();

    bands
        .iter()
        .enumerate()
        .filter_map(|(index, &amplitude)| {
            if !amplitude.is_finite() || amplitude <= noise_floor {
                return None;
            }

            let amplitude = amplitude.clamp(0.0, 1.0);
            Some(make_pulse(
                base + (index as u64 % cap),
                amplitude,
                amplitude,
                tick,
                PulseType::Modulatory,
                Rate::default(),
            ))
        })
        .collect()
}

/// Convert waveform samples to pulses using RMS amplitude and zero-crossing rate.
pub fn audio_waveform_to_pulses(
    samples: &[f32],
    window_size: usize,
    tick: u64,
) -> Vec<Pulse> {
    if window_size == 0 {
        return Vec::new();
    }

    let base = Modality::Audio.sensory_offset();
    let cap = Modality::Audio.capacity();
    let mut pulses = Vec::new();

    for (window_index, window) in samples.chunks(window_size).enumerate() {
        let finite_count = window.iter().filter(|sample| sample.is_finite()).count();
        if finite_count == 0 {
            continue;
        }

        let square_sum: f32 = window
            .iter()
            .filter(|sample| sample.is_finite())
            .map(|sample| *sample * *sample)
            .sum();
        let rms = (square_sum / finite_count as f32).sqrt();
        if !rms.is_finite() || rms < 0.01 {
            continue;
        }

        let crossing_count = window
            .windows(2)
            .filter(|pair| {
                pair[0].is_finite()
                    && pair[1].is_finite()
                    && ((pair[0] >= 0.0) != (pair[1] >= 0.0))
            })
            .count();
        let zcr = crossing_count as f32 / window.len().saturating_sub(1).max(1) as f32;

        let offset = ((window_index as u64).wrapping_add((zcr * 64.0) as u64)) % cap;
        let mut rate = Rate::default();
        rate.temporal_phase = zcr.clamp(0.0, 1.0);

        pulses.push(make_pulse(
            base + offset,
            rms,
            rms,
            tick,
            PulseType::Modulatory,
            rate,
        ));
    }

    pulses
}

/// Encode Unicode scalar values as character-level pulses — no tokenization.
pub fn text_to_pulses(text: &str, tick: u64) -> Vec<Pulse> {
    let base = Modality::Text.sensory_offset();
    let cap = Modality::Text.capacity();

    text.chars()
        .enumerate()
        .map(|(index, character)| {
            let scalar = character as u32;
            let offset = scalar as u64 % cap;
            let amplitude = (scalar % 256) as f32 / 255.0;
            let phase = ((scalar >> 8) % 16) as f32 / 15.0;
            let pulse_amplitude = amplitude.max(0.1) * (0.5 + phase * 0.5);

            let mut rate = Rate::default();
            rate.temporal_phase = phase;
            rate.sequence_position = index as u32;

            make_pulse(
                base + offset,
                amplitude.max(0.1),
                pulse_amplitude,
                tick.saturating_add(index as u64),
                PulseType::Value,
                rate,
            )
        })
        .collect()
}

/// Encode overlapping character trigrams as sequence-sensitive pulses.
pub fn text_trigrams_to_pulses(text: &str, tick: u64) -> Vec<Pulse> {
    let base = Modality::Text.sensory_offset();
    let cap = Modality::Text.capacity();
    let characters: Vec<char> = text.chars().collect();
    let mut pulses = Vec::new();

    for index in 0..characters.len().saturating_sub(2) {
        let first = characters[index] as u64;
        let second = characters[index + 1] as u64;
        let third = characters[index + 2] as u64;
        let hash = first
            .wrapping_mul(31)
            .wrapping_add(second.wrapping_mul(17))
            .wrapping_add(third);
        let amplitude = 0.7 - (index as f32 * 0.005).min(0.5);

        let mut rate = Rate::default();
        rate.sequence_position = index as u32;

        pulses.push(make_pulse(
            base + (hash % cap),
            amplitude,
            amplitude,
            tick.saturating_add(index as u64),
            PulseType::Value,
            rate,
        ));
    }

    pulses
}

/// Convert a 2D pressure map (e.g. touchscreen or tactile sensor) to pulses.
pub fn touch_pressure_to_pulses(
    pressure: &[f32],
    width: usize,
    tick: u64,
    noise_floor: f32,
) -> Vec<Pulse> {
    if width == 0 {
        return Vec::new();
    }

    let base = Modality::Touch.sensory_offset();
    let cap = Modality::Touch.capacity();
    let row_count = pressure.len().saturating_add(width - 1) / width;

    pressure
        .iter()
        .enumerate()
        .filter_map(|(index, &value)| {
            if !value.is_finite() || value <= noise_floor {
                return None;
            }

            let mut rate = Rate::default();
            rate.spatial_x = (index % width) as f32 / width as f32;
            rate.spatial_y = (index / width) as f32 / row_count.max(1) as f32;

            Some(make_pulse(
                base + (index as u64 % cap),
                value,
                value,
                tick,
                PulseType::Value,
                rate,
            ))
        })
        .collect()
}

/// Encode an internal state vector (joint angles, chemical levels, etc.).
pub fn proprioceptive_to_pulses(state: &[f32], tick: u64) -> Vec<Pulse> {
    let base = Modality::Proprioceptive.sensory_offset();
    let cap = Modality::Proprioceptive.capacity();

    state
        .iter()
        .enumerate()
        .filter_map(|(index, &value)| {
            if !value.is_finite() {
                return None;
            }

            Some(make_pulse(
                base + (index as u64 % cap),
                value,
                value.abs(),
                tick,
                PulseType::Modulatory,
                Rate::default(),
            ))
        })
        .collect()
}

/// Encode an arbitrary float vector into a custom modality region.
pub fn custom_to_pulses(
    channel: u8,
    data: &[f32],
    tick: u64,
    noise_floor: f32,
) -> Vec<Pulse> {
    let modality = Modality::Custom(channel);
    let base = modality.sensory_offset();
    let cap = modality.capacity();

    data.iter()
        .enumerate()
        .filter_map(|(index, &value)| {
            if !value.is_finite() || value.abs() <= noise_floor {
                return None;
            }

            Some(make_pulse(
                base + (index as u64 % cap),
                value,
                value.abs(),
                tick,
                PulseType::Value,
                Rate::default(),
            ))
        })
        .collect()
}

/// A bundle of pulses from multiple modalities for cross-modal temporal binding.
#[derive(Debug, Clone, Default)]
pub struct MultiModalInput {
    pub tick: u64,
    pub pulses_by_modality: Vec<(Modality, Vec<Pulse>)>,
}

impl MultiModalInput {
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            pulses_by_modality: Vec::new(),
        }
    }

    pub fn add(&mut self, modality: Modality, pulses: Vec<Pulse>) -> &mut Self {
        self.pulses_by_modality.push((modality, pulses));
        self
    }

    /// Flatten into one pulse vector for injection into the engine.
    pub fn into_flat(self) -> Vec<Pulse> {
        self.pulses_by_modality
            .into_iter()
            .flat_map(|(_, pulses)| pulses)
            .collect()
    }

    pub fn total_pulses(&self) -> usize {
        self.pulses_by_modality
            .iter()
            .map(|(_, pulses)| pulses.len())
            .sum()
    }

    pub fn modality_count(&self) -> usize {
        self.pulses_by_modality.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modality_regions_dont_overlap() {
        let modalities = [
            Modality::Vision,
            Modality::Audio,
            Modality::Text,
            Modality::Touch,
            Modality::Proprioceptive,
        ];

        for (first_index, first_modality) in modalities.iter().enumerate() {
            for second_modality in modalities.iter().skip(first_index + 1) {
                let a_start = first_modality.sensory_offset();
                let a_end = a_start + first_modality.capacity();
                let b_start = second_modality.sensory_offset();
                let b_end = b_start + second_modality.capacity();

                assert!(
                    a_end <= b_start || b_end <= a_start,
                    "Overlap between {} and {}",
                    first_modality.name(),
                    second_modality.name()
                );
            }
        }
    }

    #[test]
    fn test_grayscale_image_encoding() {
        let pixels = vec![0u8, 128, 255, 10, 200, 50];
        let pulses = image_grayscale_to_pulses(&pixels, 3, 2, 100, 20);
        assert_eq!(pulses.len(), 4);

        for pulse in &pulses {
            assert!(pulse.intensity > 0.0 && pulse.intensity <= 1.0);
            assert_eq!(pulse.created_tick, 100);
            assert!(pulse.origin.0 >= Modality::Vision.sensory_offset());
        }
    }

    #[test]
    fn test_rgb_image_encoding() {
        let rgb = vec![255u8, 0, 0, 0, 255, 0, 0, 0, 255];
        let pulses = image_rgb_to_pulses(&rgb, 3, 1, 0, 10);
        assert_eq!(pulses.len(), 3);
        assert_eq!(pulses[0].rate.channel, 0);
        assert_eq!(pulses[1].rate.channel, 1);
        assert_eq!(pulses[2].rate.channel, 2);
    }

    #[test]
    fn test_audio_bands_encoding() {
        let bands = vec![0.9, 0.0, 0.5, 0.001, 0.7];
        let pulses = audio_bands_to_pulses(&bands, 0, 0.01);
        assert_eq!(pulses.len(), 3);

        for pulse in &pulses {
            let base = Modality::Audio.sensory_offset();
            assert!(pulse.origin.0 >= base);
            assert!(pulse.origin.0 < base + Modality::Audio.capacity());
            assert_eq!(pulse.pulse_type, PulseType::Modulatory);
        }
    }

    #[test]
    fn test_audio_waveform_encoding() {
        let samples = vec![0.1, -0.3, 0.8, -0.9, 0.7, -0.2];
        let pulses = audio_waveform_to_pulses(&samples, 3, 9);
        assert_eq!(pulses.len(), 2);
        assert!(pulses.iter().all(|pulse| pulse.rate.temporal_phase > 0.0));
    }

    #[test]
    fn test_text_character_encoding() {
        let pulses = text_to_pulses("hi", 50);
        assert_eq!(pulses.len(), 2);
        assert_eq!(pulses[0].created_tick, 50);
        assert_eq!(pulses[1].created_tick, 51);
        assert_ne!(pulses[0].origin, pulses[1].origin);
        assert_eq!(pulses[1].rate.sequence_position, 1);
    }

    #[test]
    fn test_text_trigram_encoding() {
        assert_eq!(text_trigrams_to_pulses("hello", 0).len(), 3);
    }

    #[test]
    fn test_text_trigrams_short_string() {
        assert!(text_trigrams_to_pulses("hi", 0).is_empty());
    }

    #[test]
    fn test_touch_encoding() {
        let pressure = vec![0.0, 0.8, 0.3, 0.001];
        let pulses = touch_pressure_to_pulses(&pressure, 2, 0, 0.01);
        assert_eq!(pulses.len(), 2);
        assert!(pulses[0].rate.spatial_x > 0.0);
    }

    #[test]
    fn test_proprioceptive_encoding() {
        let state = vec![0.5, -0.3, 0.9];
        let pulses = proprioceptive_to_pulses(&state, 0);
        assert_eq!(pulses.len(), 3);
        assert!(pulses[1].value < 0.0);
    }

    #[test]
    fn test_custom_modality() {
        let data = vec![0.1, 0.9, 0.0];
        let pulses = custom_to_pulses(5, &data, 0, 0.05);
        assert_eq!(pulses.len(), 2);

        let base = Modality::Custom(5).sensory_offset();
        assert!(pulses.iter().all(|pulse| pulse.origin.0 >= base));
    }

    #[test]
    fn test_multimodal_bundle() {
        let mut bundle = MultiModalInput::new(42);
        bundle
            .add(
                Modality::Vision,
                image_grayscale_to_pulses(&[255, 128], 2, 1, 42, 0),
            )
            .add(Modality::Audio, audio_bands_to_pulses(&[0.5, 0.8], 42, 0.0))
            .add(Modality::Text, text_to_pulses("x", 42));

        assert_eq!(bundle.modality_count(), 3);
        assert!(bundle.total_pulses() > 0);
        assert!(!bundle.into_flat().is_empty());
    }

    #[test]
    fn test_amplitude_clamping() {
        let pulse = make_pulse(0, 2.5, 2.5, 0, PulseType::Value, Rate::default());
        assert_eq!(pulse.intensity, 1.0);
        assert_eq!(pulse.value, 2.5);

        let negative = make_pulse(0, -0.5, -0.5, 0, PulseType::Value, Rate::default());
        assert_eq!(negative.intensity, 0.0);
        assert_eq!(negative.value, -0.5);
    }

    #[test]
    fn test_zero_dimensions_and_invalid_windows_are_safe() {
        assert!(image_grayscale_to_pulses(&[255], 0, 1, 0, 0).is_empty());
        assert!(image_rgb_to_pulses(&[255, 0, 0], 1, 0, 0, 0).is_empty());
        assert!(touch_pressure_to_pulses(&[0.5], 0, 0, 0.0).is_empty());
        assert!(audio_waveform_to_pulses(&[0.5], 0, 0).is_empty());
    }
}
