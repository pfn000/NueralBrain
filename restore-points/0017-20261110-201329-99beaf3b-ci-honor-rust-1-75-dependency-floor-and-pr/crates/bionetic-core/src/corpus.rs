//! Corpus: a brain region containing organized dot populations.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{CorpusId, DotId, GateId, ClumpId, BundleId, Dot, Gate, ChemicalState};
use crate::clump::Clump;
use crate::bundle::Bundle;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CorpusType {
    Sensory,
    Prefrontal,
    Association,
    Hippocampus,
    Reward,
    Motor,
    LongTermMemory,
    Dream,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Corpus {
    pub id: CorpusId,
    pub name: String,
    pub corpus_type: CorpusType,
    pub dots: HashMap<DotId, Dot>,
    pub gates: HashMap<GateId, Gate>,
    pub clumps: HashMap<ClumpId, Clump>,
    pub bundles: HashMap<BundleId, Bundle>,
    pub ambient_chemicals: ChemicalState,
    pub activation: f64,
    pub cycle_count: u64,
}

impl Corpus {
    pub fn new(id: CorpusId, name: &str, corpus_type: CorpusType) -> Self {
        Corpus {
            id,
            name: name.to_string(),
            corpus_type,
            dots: HashMap::new(),
            gates: HashMap::new(),
            clumps: HashMap::new(),
            bundles: HashMap::new(),
            ambient_chemicals: ChemicalState::resting(),
            activation: 0.5,
            cycle_count: 0,
        }
    }

    pub fn dot_count(&self) -> usize {
        self.dots.len()
    }

    pub fn connection_count(&self) -> usize {
        self.dots.values().map(|d| d.branches.len()).sum()
    }

    pub fn active_dots(&self, current_tick: u64, window: u64) -> Vec<DotId> {
        self.dots.values()
            .filter(|d| d.fired_recently(current_tick, window))
            .map(|d| d.id)
            .collect()
    }
}

impl CorpusType {
    pub fn label(&self) -> &str {
        match self {
            CorpusType::Sensory => "Sensory",
            CorpusType::Prefrontal => "Prefrontal",
            CorpusType::Association => "Association",
            CorpusType::Hippocampus => "Hippocampus",
            CorpusType::Reward => "Reward",
            CorpusType::Motor => "Motor",
            CorpusType::LongTermMemory => "LongTermMemory",
            CorpusType::Dream => "Dream",
        }
    }
}
