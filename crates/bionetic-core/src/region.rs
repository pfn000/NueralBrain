//! Region state definitions shared by the core and runtime controller.

use serde::{Deserialize, Serialize};
use crate::ids::CorpusId;

/// Runtime state of one corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionState {
    Active,
    Resting,
    Dormant,
    Enhancing,
    Offline,
}

impl RegionState {
    pub fn accepts_external_pulses(self) -> bool {
        matches!(self, Self::Active | Self::Resting | Self::Dormant)
    }

    pub fn is_ticked(self) -> bool {
        matches!(self, Self::Active | Self::Resting | Self::Enhancing)
    }

    pub fn tick_divisor(self) -> u32 {
        match self {
            Self::Active | Self::Enhancing => 1,
            Self::Resting => 10,
            Self::Dormant | Self::Offline => 0,
        }
    }

    pub fn is_maintenance_eligible(self) -> bool {
        matches!(self, Self::Resting | Self::Dormant | Self::Enhancing)
    }
}

/// Public snapshot of one corpus' region state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionStatus {
    pub corpus: CorpusId,
    pub state: RegionState,
    pub last_transition_tick: u64,
}
