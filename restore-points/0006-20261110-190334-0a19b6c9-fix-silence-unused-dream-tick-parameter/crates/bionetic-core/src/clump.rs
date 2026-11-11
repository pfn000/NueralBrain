//! Clump: temporary gathering of dots for collaborative reasoning.

use serde::{Serialize, Deserialize};
use crate::{ClumpId, DotId};

#[derive(Debug, Serialize, Deserialize)]
pub struct Clump {
    pub id: ClumpId,
    pub member_dots: Vec<DotId>,
    pub context: ClumpContext,
    pub formed_tick: u64,
    pub coherence: f64,
    pub resolved: bool,
    pub output: Option<ClumpOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClumpContext {
    pub trigger: String,
    pub intent: Option<String>,
    pub meanings: Vec<String>,
    pub related_contexts: Vec<ClumpId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClumpOutput {
    pub resolution: String,
    pub confidence: f64,
    pub value: f64,
    pub reasoning_trace: Vec<DotId>,
}

impl Clump {
    pub fn new(id: ClumpId, members: Vec<DotId>, trigger: String, tick: u64) -> Self {
        Clump {
            id,
            member_dots: members,
            context: ClumpContext {
                trigger,
                intent: None,
                meanings: Vec::new(),
                related_contexts: Vec::new(),
            },
            formed_tick: tick,
            coherence: 0.5,
            resolved: false,
            output: None,
        }
    }

    pub fn age(&self, current_tick: u64) -> u64 {
        current_tick.saturating_sub(self.formed_tick)
    }

    pub fn is_stale(&self, current_tick: u64, max_age: u64) -> bool {
        !self.resolved && self.age(current_tick) > max_age
    }

    pub fn resolve(&mut self, output: ClumpOutput) {
        self.output = Some(output);
        self.resolved = true;
    }
}
