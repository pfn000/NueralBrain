//! Gates: signal filtering, routing, and control.
//!
//! Gate Types:
//! - Soft: probabilistic, learning-oriented
//! - Hard: deterministic, enforcement
//! - Safe: protective, prevents cascades
//! - Value: evaluative, memory consolidation
//! - Rate: temporal, frequency control
//! - Master: controls entire bundle sections
//! - Trans: bidirectional corpus-to-corpus

use serde::{Serialize, Deserialize};
use crate::{GateId, ChemicalState, ChemicalDelta, ChemicalType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gate {
    pub id: GateId,
    pub gate_type: GateType,
    /// How open this gate is (0.0 = closed, 1.0 = fully open)
    pub aperture: f64,
    /// Conditions for passage
    pub conditions: Vec<GateCondition>,
    /// Minimum signal value to pass
    pub value_threshold: f64,
    /// Maximum signal rate allowed (pulses per second)
    pub rate_limit: f64,
    /// How this gate modifies passing signals
    pub transform: GateTransform,
    /// Is this gate locked by hard override?
    pub locked: bool,
    /// Statistics
    pub pass_count: u64,
    pub block_count: u64,
    /// Recent pass timestamps for rate limiting
    pub recent_passes: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GateType {
    Soft,
    Hard,
    Safe,
    Value,
    Rate,
    Master,
    Trans,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GateCondition {
    MinValue(f64),
    MaxValue(f64),
    IntensityRange(f64, f64),
    MinFireCount(u64),
    ChemicalRange {
        chemical: ChemicalType,
        min: f64,
        max: f64,
    },
    CooldownTicks(u64),
    MinSuccessRate(f64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GateTransform {
    PassThrough,
    Amplify(f64),
    Attenuate(f64),
    Invert,
    InjectChemical(ChemicalDelta),
}

#[derive(Debug)]
pub enum GateResult {
    Passed {
        modified_value: f64,
        chemical_injection: Option<ChemicalDelta>,
    },
    Blocked,
    RateLimited,
}

impl Gate {
    pub fn new(gate_type: GateType) -> Self {
        let id = GateId::next();
        Gate {
            id,
            gate_type,
            aperture: 0.5,
            conditions: Vec::new(),
            value_threshold: 0.0,
            rate_limit: 1000.0,
            transform: GateTransform::PassThrough,
            locked: false,
            pass_count: 0,
            block_count: 0,
            recent_passes: Vec::new(),
        }
    }

    pub fn soft() -> Self { Self::new(GateType::Soft) }
    pub fn hard() -> Self { Self::new(GateType::Hard) }
    pub fn safe() -> Self { Self::new(GateType::Safe) }
    pub fn value_gate() -> Self { Self::new(GateType::Value) }
    pub fn rate_gate() -> Self { Self::new(GateType::Rate) }
    pub fn master() -> Self { Self::new(GateType::Master) }
    pub fn trans() -> Self { Self::new(GateType::Trans) }

    /// Evaluate whether a pulse value should pass through
    pub fn evaluate(
        &mut self,
        pulse_value: f64,
        pulse_intensity: f64,
        target_chemicals: &ChemicalState,
        target_fire_count: u64,
        current_tick: u64,
    ) -> GateResult {
        if self.locked {
            self.block_count += 1;
            return GateResult::Blocked;
        }

        // Check value threshold
        if pulse_value.abs() < self.value_threshold {
            self.block_count += 1;
            return GateResult::Blocked;
        }

        // Check conditions
        for condition in &self.conditions {
            match condition {
                GateCondition::MinValue(min) => {
                    if pulse_value < *min {
                        self.block_count += 1;
                        return GateResult::Blocked;
                    }
                }
                GateCondition::MaxValue(max) => {
                    if pulse_value > *max {
                        self.block_count += 1;
                        return GateResult::Blocked;
                    }
                }
                GateCondition::IntensityRange(min, max) => {
                    if pulse_intensity < *min || pulse_intensity > *max {
                        self.block_count += 1;
                        return GateResult::Blocked;
                    }
                }
                GateCondition::MinFireCount(min) => {
                    if target_fire_count < *min {
                        self.block_count += 1;
                        return GateResult::Blocked;
                    }
                }
                GateCondition::ChemicalRange { chemical, min, max } => {
                    let level = target_chemicals.get(chemical);
                    if level < *min || level > *max {
                        self.block_count += 1;
                        return GateResult::Blocked;
                    }
                }
                GateCondition::CooldownTicks(cooldown) => {
                    if let Some(&last) = self.recent_passes.last() {
                        if current_tick - last < *cooldown {
                            self.block_count += 1;
                            return GateResult::RateLimited;
                        }
                    }
                }
                GateCondition::MinSuccessRate(_) => {
                    // Evaluated at branch level
                }
            }
        }

        // Apply transform
        let modified_value = match &self.transform {
            GateTransform::PassThrough => pulse_value * self.aperture,
            GateTransform::Amplify(factor) => pulse_value * factor * self.aperture,
            GateTransform::Attenuate(factor) => pulse_value * factor * self.aperture,
            GateTransform::Invert => -pulse_value * self.aperture,
            GateTransform::InjectChemical(_) => pulse_value * self.aperture,
        };

        let chemical_injection = match &self.transform {
            GateTransform::InjectChemical(delta) => Some(delta.clone()),
            _ => None,
        };

        self.pass_count += 1;
        self.recent_passes.push(current_tick);
        // Keep only last 100 timestamps
        if self.recent_passes.len() > 100 {
            self.recent_passes.remove(0);
        }

        GateResult::Passed {
            modified_value,
            chemical_injection,
        }
    }

    /// Adjust aperture (used in learning)
    pub fn open(&mut self, amount: f64) {
        self.aperture = (self.aperture + amount).clamp(0.0, 1.0);
    }

    pub fn close(&mut self, amount: f64) {
        self.aperture = (self.aperture - amount).clamp(0.0, 1.0);
    }
}
