//! Rate: speed, seed, value, intensity, and sensory-position metadata.

use serde::{Deserialize, Serialize};

/// Signal metadata shared by pulse routing, gates, and multi-modal encoders.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Rate {
    /// How fast the signal/process operates.
    pub speed: f64,
    /// Seed for deterministic reproducibility.
    pub seed: u64,
    /// Computed importance/worth.
    pub value: f64,
    /// Force/magnitude.
    pub intensity: f64,

    /// Spatial X coordinate in the source signal (normalized 0.0–1.0).
    pub spatial_x: f32,
    /// Spatial Y coordinate in the source signal (normalized 0.0–1.0).
    pub spatial_y: f32,
    /// Temporal phase (0.0–1.0) for rhythmic signals.
    pub temporal_phase: f32,
    /// Channel index (RGB = 0,1,2; stereo = 0,1; etc.).
    pub channel: u8,
    /// Position in a sequence (character, word, frame, etc.).
    pub sequence_position: u32,
}

impl Rate {
    pub fn new(speed: f64, seed: u64, value: f64, intensity: f64) -> Self {
        Self {
            speed,
            seed,
            value,
            intensity,
            ..Self::default()
        }
    }

    pub fn default_signal() -> Self {
        Self {
            speed: 1.0,
            seed: 0,
            value: 0.5,
            intensity: 0.5,
            spatial_x: 0.0,
            spatial_y: 0.0,
            temporal_phase: 0.0,
            channel: 0,
            sequence_position: 0,
        }
    }

    /// Composite score for gate decisions.
    pub fn composite_score(&self) -> f64 {
        (self.value * 0.4) + (self.intensity * 0.4) + (self.speed * 0.2)
    }

    /// Decay rate over time.
    pub fn decay(&mut self, elapsed_seconds: f64, decay_constant: f64) {
        let factor = (-decay_constant * elapsed_seconds).exp();
        self.intensity *= factor;
        self.value *= (factor + 1.0) / 2.0;
    }

    /// Boost from reward/reinforcement.
    pub fn reinforce(&mut self, reward: f64) {
        self.value = (self.value + reward * 0.1).min(100.0);
        self.intensity = (self.intensity + reward * 0.05).min(100.0);
    }
}

impl Default for Rate {
    fn default() -> Self {
        Self::default_signal()
    }
}
