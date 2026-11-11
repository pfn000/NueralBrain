//! Dot: the fundamental computational unit (neuron analog).

use serde::{Serialize, Deserialize};
use crate::{DotId, GateId, CorpusId, ClumpId, ChemicalState};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dot {
    pub id: DotId,

    // === ELECTRICAL STATE ===
    pub potential: f64,
    pub resting_potential: f64,
    pub threshold: f64,
    pub base_threshold: f64,
    pub refractory: bool,
    pub refractory_remaining: u32,
    pub refractory_duration: u32,

    // === CHEMICAL STATE ===
    pub chemicals: ChemicalState,

    // === STRUCTURAL CONNECTIONS ===
    pub endings: Vec<Ending>,
    pub branches: Vec<Branch>,

    // === MEMORY & HISTORY ===
    pub fire_count: u64,
    pub recent_fire_ticks: Vec<u64>,
    pub value: f64,
    pub fatigue: f64,

    // === METADATA ===
    pub corpus: CorpusId,
    pub born_tick: u64,
    pub tags: Vec<String>,
    pub clump_id: Option<ClumpId>,
}

/// An incoming connection point (dendrite analog)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ending {
    pub id: u32,
    pub source_dot: DotId,
    pub gate: GateId,
    pub weight: f64,
    pub usage_count: u64,
    pub last_active_tick: Option<u64>,
}

/// An outgoing connection (axon branch analog)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Branch {
    pub id: u32,
    pub target_dot: DotId,
    pub target_corpus: CorpusId,
    pub gate: GateId,
    pub strength: f64,
    pub delay_ticks: u32,
    pub success_rate: f64,
    pub transmission_count: u64,
}

impl Dot {
    pub fn new(id: DotId, corpus: CorpusId, born_tick: u64) -> Self {
        Dot {
            id,
            potential: -70.0,
            resting_potential: -70.0,
            threshold: -55.0,
            base_threshold: -55.0,
            refractory: false,
            refractory_remaining: 0,
            refractory_duration: 2,
            chemicals: ChemicalState::resting(),
            endings: Vec::new(),
            branches: Vec::new(),
            fire_count: 0,
            recent_fire_ticks: Vec::new(),
            value: 0.0,
            fatigue: 0.0,
            corpus,
            born_tick,
            tags: Vec::new(),
            clump_id: None,
        }
    }

    /// Check if this dot should fire
    pub fn should_fire(&self) -> bool {
        !self.refractory && self.potential > self.threshold
    }

    /// Execute firing: reset potential, enter refractory, update stats
    pub fn fire(&mut self, current_tick: u64) {
        self.fire_count += 1;
        self.recent_fire_ticks.push(current_tick);

        // Keep only last 100 fire ticks
        if self.recent_fire_ticks.len() > 100 {
            self.recent_fire_ticks.remove(0);
        }

        // Hyperpolarize
        self.potential = self.resting_potential - 10.0;

        // Enter refractory
        self.refractory = true;
        self.refractory_remaining = self.refractory_duration;

        // Accumulate fatigue
        self.fatigue = (self.fatigue + 0.01).min(1.0);
    }

    /// Decay potential toward resting
    pub fn decay_potential(&mut self, rate: f64) {
        self.potential += (self.resting_potential - self.potential) * rate;
    }

    /// Update refractory period
    pub fn tick_refractory(&mut self) {
        if self.refractory {
            if self.refractory_remaining > 0 {
                self.refractory_remaining -= 1;
            } else {
                self.refractory = false;
            }
        }
    }

    /// Recalculate threshold based on chemicals and fatigue
    pub fn update_threshold(&mut self) {
        self.threshold = self.base_threshold;
        // Cortisol raises threshold (harder to fire under stress)
        self.threshold += self.chemicals.cortisol * 5.0;
        // GABA inhibits
        self.threshold += self.chemicals.gaba * 3.0;
        // Glutamate excites (lowers threshold)
        self.threshold -= self.chemicals.glutamate * 2.0;
        // Fatigue raises threshold
        self.threshold += self.fatigue * 10.0;
        // Clamp
        self.threshold = self.threshold.clamp(-65.0, -30.0);
    }

    /// Recover fatigue
    pub fn recover_fatigue(&mut self, amount: f64) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }

    /// Check if this dot fired recently (within window)
    pub fn fired_recently(&self, current_tick: u64, window: u64) -> bool {
        self.recent_fire_ticks.last()
            .map(|&t| current_tick - t <= window)
            .unwrap_or(false)
    }

    /// Get firing rate (fires per N ticks)
    pub fn firing_rate(&self, current_tick: u64, window: u64) -> f64 {
        let count = self.recent_fire_ticks.iter()
            .filter(|&&t| current_tick - t <= window)
            .count();
        count as f64 / window as f64
    }

    /// Add an incoming connection
    pub fn add_ending(&mut self, source: DotId, gate: GateId, weight: f64) {
        let id = self.endings.len() as u32;
        self.endings.push(Ending {
            id,
            source_dot: source,
            gate,
            weight,
            usage_count: 0,
            last_active_tick: None,
        });
    }

    /// Add an outgoing connection
    pub fn add_branch(
        &mut self,
        target: DotId,
        target_corpus: CorpusId,
        gate: GateId,
        strength: f64,
    ) {
        let id = self.branches.len() as u32;
        self.branches.push(Branch {
            id,
            target_dot: target,
            target_corpus,
            gate,
            strength,
            delay_ticks: 1,
            success_rate: 0.5,
            transmission_count: 0,
        });
    }
}
