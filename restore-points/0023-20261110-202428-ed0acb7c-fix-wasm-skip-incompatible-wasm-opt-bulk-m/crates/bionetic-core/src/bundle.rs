//! Bundle: organized group of branches forming a pathway.

use serde::{Serialize, Deserialize};
use crate::{BundleId, DotId, GateId};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub id: BundleId,
    pub members: Vec<BundleMember>,
    pub safe_gates: Vec<GateId>,
    pub strength: f64,
    pub usage_count: u64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMember {
    pub source_dot: DotId,
    pub target_dot: DotId,
    pub branch_id: u32,
}

impl Bundle {
    pub fn new(id: BundleId) -> Self {
        Bundle {
            id,
            members: Vec::new(),
            safe_gates: Vec::new(),
            strength: 0.5,
            usage_count: 0,
            label: None,
        }
    }
}
