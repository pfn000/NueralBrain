//! Trans-Binary: bidirectional simultaneous communication channels.

use serde::{Serialize, Deserialize};
use crate::{ChannelId, DotId, CorpusId, BundleId, Pulse, ChemicalState, ClumpId, Rate};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransBinaryChannel {
    pub id: ChannelId,
    pub corpus_a: CorpusId,
    pub corpus_b: CorpusId,
    pub bundle_id: BundleId,
    pub active: bool,
    pub forward_count: u64,
    pub backward_count: u64,
    pub simultaneous_count: u64,
    /// Pending signals in each direction
    pub forward_queue: Vec<TransSignal>,
    pub backward_queue: Vec<TransSignal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransSignal {
    pub pulse: Pulse,
    pub register: RegisterData,
    pub feed: FeedData,
    pub tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterData {
    pub source_id: DotId,
    pub potential: f64,
    pub chemical_state: ChemicalState,
    pub fire_count: u64,
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeedData {
    pub active_context: Option<ClumpId>,
    pub priority: f64,
    pub rate: Rate,
}

impl TransBinaryChannel {
    pub fn new(corpus_a: CorpusId, corpus_b: CorpusId) -> Self {
        TransBinaryChannel {
            id: ChannelId::next(),
            corpus_a,
            corpus_b,
            bundle_id: BundleId::next(),
            active: true,
            forward_count: 0,
            backward_count: 0,
            simultaneous_count: 0,
            forward_queue: Vec::new(),
            backward_queue: Vec::new(),
        }
    }

    pub fn send_forward(&mut self, signal: TransSignal) {
        if self.active {
            self.forward_queue.push(signal);
            self.forward_count += 1;
        }
    }

    pub fn send_backward(&mut self, signal: TransSignal) {
        if self.active {
            self.backward_queue.push(signal);
            self.backward_count += 1;
        }
    }

    pub fn drain_forward(&mut self) -> Vec<TransSignal> {
        std::mem::take(&mut self.forward_queue)
    }

    pub fn drain_backward(&mut self) -> Vec<TransSignal> {
        std::mem::take(&mut self.backward_queue)
    }
}
