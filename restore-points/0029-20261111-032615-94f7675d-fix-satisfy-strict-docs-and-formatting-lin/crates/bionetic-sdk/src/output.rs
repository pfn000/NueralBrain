//! High-level output representation.

use bionetic_core::chemicals::ChemicalSnapshot;
use serde::{Deserialize, Serialize};

/// The result of a `think()` call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrainOutput {
    /// Motor-cortex output values (one per motor dot).
    pub motor: Vec<f32>,
    /// Current tick after processing.
    pub tick: u64,
    /// Chemical state snapshot after processing.
    pub chemicals: ChemicalSnapshot,
    /// Number of dots that fired this tick.
    pub active_dots: usize,
    /// Number of pulses that propagated this tick.
    pub propagated_pulses: usize,
}

impl BrainOutput {
    /// The strongest single output value (useful for classification).
    pub fn peak(&self) -> Option<(usize, f32)> {
        self.motor
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, &v)| (i, v))
    }

    /// Total output energy (sum of absolute values).
    pub fn total_energy(&self) -> f32 {
        self.motor.iter().map(|v| v.abs()).sum()
    }

    /// Normalize output to [0.0, 1.0] range.
    pub fn normalized(&self) -> Vec<f32> {
        let max = self.motor.iter().cloned().fold(0.0f32, f32::max);
        if max <= 0.0 {
            return self.motor.clone();
        }
        self.motor.iter().map(|v| v / max).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peak() {
        let out = BrainOutput {
            motor: vec![0.1, 0.5, 0.3, 0.9, 0.2],
            ..Default::default()
        };
        let (idx, val) = out.peak().unwrap();
        assert_eq!(idx, 3);
        assert!((val - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_total_energy() {
        let out = BrainOutput {
            motor: vec![0.5, -0.3, 0.2],
            ..Default::default()
        };
        assert!((out.total_energy() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalized() {
        let out = BrainOutput {
            motor: vec![0.5, 1.0, 0.25],
            ..Default::default()
        };
        let n = out.normalized();
        assert_eq!(n, vec![0.5, 1.0, 0.25]);
    }

    #[test]
    fn test_empty_peak() {
        let out = BrainOutput::default();
        assert!(out.peak().is_none());
    }
}