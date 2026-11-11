//! High-level input batching.

use bionetic_core::ids::{CorpusId, DotId};
use bionetic_core::pulse::{Pulse, PulseType};

/// A convenient batch of inputs targeting a specific corpus.
#[derive(Debug, Clone)]
pub struct InputBatch {
    /// Which corpus to inject into.
    pub target_corpus: CorpusId,
    /// Value pulses (normal signal).
    pub values: Vec<f32>,
    /// Reward pulses.
    pub rewards: Vec<f32>,
    /// Inhibitory pulses.
    pub inhibits: Vec<f32>,
    /// Modulatory pulses.
    pub modulations: Vec<f32>,
}

impl InputBatch {
    /// Create a new batch targeting the given corpus.
    pub fn new(target_corpus: CorpusId) -> Self {
        Self { target_corpus, values: Vec::new(), rewards: Vec::new(), inhibits: Vec::new(), modulations: Vec::new() }
    }
    /// Add value pulses.
    pub fn with_values(mut self, values: &[f32]) -> Self { self.values.extend_from_slice(values); self }
    /// Add reward pulses.
    pub fn with_rewards(mut self, values: &[f32]) -> Self { self.rewards.extend_from_slice(values); self }
    /// Add inhibitory pulses.
    pub fn with_inhibits(mut self, values: &[f32]) -> Self { self.inhibits.extend_from_slice(values); self }
    /// Add modulatory pulses.
    pub fn with_modulations(mut self, values: &[f32]) -> Self { self.modulations.extend_from_slice(values); self }
    /// Total pulse count.
    pub fn total_pulses(&self) -> usize {
        self.values.len() + self.rewards.len() + self.inhibits.len() + self.modulations.len()
    }
    /// Convert this batch into pulse objects stamped with the supplied tick.
    pub fn into_pulses(self, tick: u64) -> Vec<Pulse> {
        let mut pulses = Vec::with_capacity(self.total_pulses());
        let mut index = 0u64;
        for value in self.values { pulses.push(make_pulse(index, value, tick, PulseType::Value)); index += 1; }
        for value in self.rewards { pulses.push(make_pulse(index, value, tick, PulseType::Reward)); index += 1; }
        for value in self.inhibits { pulses.push(make_pulse(index, value, tick, PulseType::Inhibitory)); index += 1; }
        for value in self.modulations { pulses.push(make_pulse(index, value, tick, PulseType::Modulatory)); index += 1; }
        pulses
    }
}

#[inline]
fn make_pulse(index: u64, strength: f32, tick: u64, pulse_type: PulseType) -> Pulse {
    let dot_id = DotId(index);
    let amplitude = strength.clamp(-1.0, 1.0);
    let mut pulse = match pulse_type {
        PulseType::Value => Pulse::value_pulse(dot_id, amplitude as f64, amplitude.abs() as f64, tick),
        PulseType::Reward => Pulse::reward_pulse(dot_id, amplitude as f64, tick),
        PulseType::Inhibitory => Pulse::inhibitory_pulse(dot_id, amplitude as f64, tick),
        other => {
            let mut pulse = Pulse::value_pulse(dot_id, amplitude as f64, amplitude.abs() as f64, tick);
            pulse.pulse_type = other;
            pulse
        }
    };
    pulse.rate.sequence_position = index.min(u32::MAX as u64) as u32;
    pulse
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_builder() {
        let batch = InputBatch::new(CorpusId(0)).with_values(&[0.1, 0.2]).with_rewards(&[0.9]);
        assert_eq!(batch.total_pulses(), 3);
    }

    #[test]
    fn test_batch_into_pulses() {
        let batch = InputBatch::new(CorpusId(1)).with_values(&[0.5]).with_inhibits(&[0.3]);
        let pulses = batch.into_pulses(42);
        assert_eq!(pulses.len(), 2);
        assert_eq!(pulses[0].created_tick, 42);
    }

    #[test]
    fn test_amplitude_clamping() {
        let pulses = InputBatch::new(CorpusId(0)).with_values(&[5.0]).into_pulses(0);
        assert_eq!(pulses[0].intensity, 1.0);
    }
}
