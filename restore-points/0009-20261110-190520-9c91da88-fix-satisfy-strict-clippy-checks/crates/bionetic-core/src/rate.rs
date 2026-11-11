//! Rate: speed, seed, value, intensity metadata for signals and processes.

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rate {
    /// How fast the signal/process operates
    pub speed: f64,
    /// Seed for deterministic reproducibility
    pub seed: u64,
    /// Computed importance/worth
    pub value: f64,
    /// Force/magnitude
    pub intensity: f64,
}

impl Rate {
    pub fn new(speed: f64, seed: u64, value: f64, intensity: f64) -> Self {
        Rate { speed, seed, value, intensity }
    }

    pub fn default_signal() -> Self {
        Rate {
            speed: 1.0,
            seed: 0,
            value: 0.5,
            intensity: 0.5,
        }
    }

    /// Composite score for gate decisions
    pub fn composite_score(&self) -> f64 {
        (self.value * 0.4) + (self.intensity * 0.4) + (self.speed * 0.2)
    }

    /// Decay rate over time
    pub fn decay(&mut self, elapsed_seconds: f64, decay_constant: f64) {
        let factor = (-decay_constant * elapsed_seconds).exp();
        self.intensity *= factor;
        self.value *= (factor + 1.0) / 2.0;
    }

    /// Boost from reward/reinforcement
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
