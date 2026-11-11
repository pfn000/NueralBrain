//! Corpus: a brain region containing organized dot populations.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{CorpusId, DotId, GateId, ClumpId, BundleId, Dot, Gate, ChemicalState};
use crate::clump::Clump;
use crate::bundle::Bundle;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CorpusType {
    /// Legacy sensory variant; new code should use the named SENSORY_IN slot.
    Sensory,
    SensoryOut,
    Vision,
    RelationalContext,
    ShortTermMemory,
    Auditory,
    SpeechMotor,
    Prefrontal,
    Association,
    Hippocampus,
    Reward,
    Motor,
    LongTermMemory,
    Dream,
}


/// Canonical semantic kind for each corpus slot. This is additive to the
/// existing CorpusType API so older data and call sites remain valid.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorpusKind {
    SensoryIn, SensoryOut, Vision, Prefrontal, Association, RelationalContext,
    ShortTermMemory, Hippocampus, LongTermMemory, Reward, Motor, Dream,
    Auditory, SpeechMotor,
}

impl CorpusKind {
    pub fn id(self) -> CorpusId {
        use crate::ids::*;
        match self {
            Self::SensoryIn => SENSORY_IN,
            Self::SensoryOut => SENSORY_OUT,
            Self::Vision => VISION,
            Self::Prefrontal => PREFRONTAL,
            Self::Association => ASSOCIATION,
            Self::RelationalContext => RELATIONAL_CONTEXT,
            Self::ShortTermMemory => SHORT_TERM_MEMORY,
            Self::Hippocampus => HIPPOCAMPUS,
            Self::LongTermMemory => LONG_TERM_MEMORY,
            Self::Reward => REWARD,
            Self::Motor => MOTOR,
            Self::Dream => DREAM,
            Self::Auditory => AUDITORY,
            Self::SpeechMotor => SPEECH_MOTOR,
        }
    }

    pub fn from_id(id: CorpusId) -> Option<Self> {
        Some(match id {
            crate::ids::SENSORY_IN => Self::SensoryIn,
            crate::ids::SENSORY_OUT => Self::SensoryOut,
            crate::ids::VISION => Self::Vision,
            crate::ids::PREFRONTAL => Self::Prefrontal,
            crate::ids::ASSOCIATION => Self::Association,
            crate::ids::RELATIONAL_CONTEXT => Self::RelationalContext,
            crate::ids::SHORT_TERM_MEMORY => Self::ShortTermMemory,
            crate::ids::HIPPOCAMPUS => Self::Hippocampus,
            crate::ids::LONG_TERM_MEMORY => Self::LongTermMemory,
            crate::ids::REWARD => Self::Reward,
            crate::ids::MOTOR => Self::Motor,
            crate::ids::DREAM => Self::Dream,
            crate::ids::AUDITORY => Self::Auditory,
            crate::ids::SPEECH_MOTOR => Self::SpeechMotor,
            _ => return None,
        })
    }

    pub fn is_reserved(self) -> bool { matches!(self, Self::Auditory | Self::SpeechMotor) }

    pub fn all() -> [Self; crate::ids::CORPUS_COUNT_TOTAL] {
        [
            Self::SensoryIn, Self::SensoryOut, Self::Vision, Self::Prefrontal,
            Self::Association, Self::RelationalContext, Self::ShortTermMemory,
            Self::Hippocampus, Self::LongTermMemory, Self::Reward, Self::Motor,
            Self::Dream, Self::Auditory, Self::SpeechMotor,
        ]
    }

    pub fn legacy_type(self) -> CorpusType {
        match self {
            Self::SensoryIn => CorpusType::Sensory,
            Self::SensoryOut => CorpusType::SensoryOut,
            Self::Vision => CorpusType::Vision,
            Self::Prefrontal => CorpusType::Prefrontal,
            Self::Association => CorpusType::Association,
            Self::RelationalContext => CorpusType::RelationalContext,
            Self::ShortTermMemory => CorpusType::ShortTermMemory,
            Self::Hippocampus => CorpusType::Hippocampus,
            Self::LongTermMemory => CorpusType::LongTermMemory,
            Self::Reward => CorpusType::Reward,
            Self::Motor => CorpusType::Motor,
            Self::Dream => CorpusType::Dream,
            Self::Auditory => CorpusType::Auditory,
            Self::SpeechMotor => CorpusType::SpeechMotor,
        }
    }
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
            CorpusType::Sensory => "SensoryIn",
            CorpusType::SensoryOut => "SensoryOut",
            CorpusType::Vision => "Vision",
            CorpusType::RelationalContext => "RelationalContext",
            CorpusType::ShortTermMemory => "ShortTermMemory",
            CorpusType::Auditory => "Auditory",
            CorpusType::SpeechMotor => "SpeechMotor",
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
