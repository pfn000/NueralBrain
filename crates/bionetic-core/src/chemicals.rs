//! Neurochemical simulation system.
//!
//! Simulated chemicals modulate dot behavior:
//! - Dopamine: reward signal, reinforces pathways
//! - Serotonin: stability, affects firing willingness
//! - Cortisol: stress/urgency, lowers threshold temporarily
//! - Acetylcholine: learning, modulates plasticity
//! - GABA: inhibitory, suppresses firing
//! - Glutamate: excitatory baseline

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChemicalState {
    pub dopamine: f64,
    pub serotonin: f64,
    pub cortisol: f64,
    pub acetylcholine: f64,
    pub gaba: f64,
    pub glutamate: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChemicalDelta {
    pub dopamine: f64,
    pub serotonin: f64,
    pub cortisol: f64,
    pub acetylcholine: f64,
    pub gaba: f64,
    pub glutamate: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChemicalType {
    Dopamine,
    Serotonin,
    Cortisol,
    Acetylcholine,
    Gaba,
    Glutamate,
}

impl ChemicalState {
    pub fn resting() -> Self {
        ChemicalState {
            dopamine: 0.1,
            serotonin: 0.5,
            cortisol: 0.1,
            acetylcholine: 0.3,
            gaba: 0.3,
            glutamate: 0.3,
        }
    }

    pub fn apply_delta(&mut self, delta: &ChemicalDelta) {
        self.dopamine = (self.dopamine + delta.dopamine).clamp(0.0, 10.0);
        self.serotonin = (self.serotonin + delta.serotonin).clamp(0.0, 10.0);
        self.cortisol = (self.cortisol + delta.cortisol).clamp(0.0, 10.0);
        self.acetylcholine = (self.acetylcholine + delta.acetylcholine).clamp(0.0, 10.0);
        self.gaba = (self.gaba + delta.gaba).clamp(0.0, 10.0);
        self.glutamate = (self.glutamate + delta.glutamate).clamp(0.0, 10.0);
    }

    pub fn decay(&mut self, rate: f64) {
        let factor = 1.0 - rate;
        self.dopamine *= factor;
        self.serotonin = self.serotonin * factor + 0.5 * rate; // serotonin returns to 0.5
        self.cortisol *= factor;
        self.acetylcholine = self.acetylcholine * factor + 0.3 * rate;
        self.gaba = self.gaba * factor + 0.3 * rate;
        self.glutamate = self.glutamate * factor + 0.3 * rate;
    }

    pub fn get(&self, chemical: &ChemicalType) -> f64 {
        match chemical {
            ChemicalType::Dopamine => self.dopamine,
            ChemicalType::Serotonin => self.serotonin,
            ChemicalType::Cortisol => self.cortisol,
            ChemicalType::Acetylcholine => self.acetylcholine,
            ChemicalType::Gaba => self.gaba,
            ChemicalType::Glutamate => self.glutamate,
        }
    }
}

impl Default for ChemicalState {
    fn default() -> Self {
        Self::resting()
    }
}

impl ChemicalDelta {
    pub fn reward(magnitude: f64) -> Self {
        ChemicalDelta {
            dopamine: magnitude * 0.8,
            serotonin: magnitude * 0.2,
            ..Default::default()
        }
    }

    pub fn discipline(magnitude: f64) -> Self {
        ChemicalDelta {
            dopamine: -magnitude * 0.3,
            cortisol: magnitude * 0.5,
            ..Default::default()
        }
    }
}

/// A read-only snapshot of chemical levels for serialization / sync.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChemicalSnapshot {
    pub dopamine: f32,
    pub serotonin: f32,
    pub cortisol: f32,
    pub acetylcholine: f32,
    pub gaba: f32,
    pub glutamate: f32,
}

impl ChemicalSnapshot {
    /// Build a snapshot from the live chemical state.
    /// (Call this method on your existing `Chemicals` or equivalent struct.)
    pub fn from_values(
        dopamine: f32,
        serotonin: f32,
        cortisol: f32,
        acetylcholine: f32,
        gaba: f32,
        glutamate: f32,
    ) -> Self {
        Self {
            dopamine,
            serotonin,
            cortisol,
            acetylcholine,
            gaba,
            glutamate,
        }
    }
}
