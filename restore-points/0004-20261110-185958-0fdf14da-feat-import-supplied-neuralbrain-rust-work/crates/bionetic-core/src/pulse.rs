//! Pulse: electrical signal that propagates between dots.

use serde::{Serialize, Deserialize};
use crate::{PulseId, DotId, ChemicalDelta};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pulse {
    pub id: PulseId,
    pub origin: DotId,
    pub value: f64,
    pub intensity: f64,
    pub speed: f64,
    pub pulse_type: PulseType,
    pub chemical_payload: ChemicalDelta,
    pub path_trace: Vec<DotId>,
    pub ttl: u32,
    pub created_tick: u64,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PulseType {
    /// V.Pulse — carries computed value forward
    Value,
    /// R.Pulse — reward/reinforcement (backward)
    Reward,
    /// I.Pulse — inhibitory, suppresses target
    Inhibitory,
    /// M.Pulse — modulatory, changes parameters
    Modulatory,
    /// Trans — bidirectional communication
    Trans,
}

impl Pulse {
    pub fn value_pulse(origin: DotId, value: f64, intensity: f64, tick: u64) -> Self {
        Pulse {
            id: PulseId::next(),
            origin,
            value,
            intensity,
            speed: 1.0,
            pulse_type: PulseType::Value,
            chemical_payload: ChemicalDelta::default(),
            path_trace: vec![origin],
            ttl: 64,
            created_tick: tick,
            seed: 0,
        }
    }

    pub fn reward_pulse(origin: DotId, reward_value: f64, tick: u64) -> Self {
        Pulse {
            id: PulseId::next(),
            origin,
            value: reward_value,
            intensity: reward_value.abs(),
            speed: 2.0,
            pulse_type: PulseType::Reward,
            chemical_payload: ChemicalDelta::reward(reward_value),
            path_trace: vec![origin],
            ttl: 128,
            created_tick: tick,
            seed: 0,
        }
    }

    pub fn inhibitory_pulse(origin: DotId, value: f64, tick: u64) -> Self {
        Pulse {
            id: PulseId::next(),
            origin,
            value: -value.abs(),
            intensity: value.abs(),
            speed: 1.5,
            pulse_type: PulseType::Inhibitory,
            chemical_payload: ChemicalDelta {
                gaba: value.abs() * 0.5,
                ..Default::default()
            },
            path_trace: vec![origin],
            ttl: 32,
            created_tick: tick,
            seed: 0,
        }
    }

    /// Record that this pulse passed through a dot
    pub fn trace(&mut self, dot_id: DotId) {
        self.path_trace.push(dot_id);
        self.ttl = self.ttl.saturating_sub(1);
    }

    /// Check if this pulse is still alive
    pub fn is_alive(&self) -> bool {
        self.ttl > 0
    }

    /// Check if this pulse already visited a dot (loop prevention)
    pub fn has_visited(&self, dot_id: DotId) -> bool {
        self.path_trace.contains(&dot_id)
    }
}
