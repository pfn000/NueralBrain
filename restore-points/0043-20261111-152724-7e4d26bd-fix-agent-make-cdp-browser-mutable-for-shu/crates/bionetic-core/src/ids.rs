//! Unique identifiers for all bionetic entities.

use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize, Ord, PartialOrd)]
pub struct DotId(pub u64);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GateId(pub u64);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct CorpusId(pub u32);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClumpId(pub u64);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct BundleId(pub u64);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PulseId(pub u64);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChannelId(pub u64);

// Global ID generators
static DOT_COUNTER: AtomicU64 = AtomicU64::new(1);
static GATE_COUNTER: AtomicU64 = AtomicU64::new(1);
static CLUMP_COUNTER: AtomicU64 = AtomicU64::new(1);
static BUNDLE_COUNTER: AtomicU64 = AtomicU64::new(1);
static PULSE_COUNTER: AtomicU64 = AtomicU64::new(1);
static CHANNEL_COUNTER: AtomicU64 = AtomicU64::new(1);

impl DotId {
    pub fn next() -> Self {
        DotId(DOT_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl GateId {
    pub fn next() -> Self {
        GateId(GATE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl ClumpId {
    pub fn next() -> Self {
        ClumpId(CLUMP_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl BundleId {
    pub fn next() -> Self {
        BundleId(BUNDLE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl PulseId {
    pub fn next() -> Self {
        PulseId(PULSE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl ChannelId {
    pub fn next() -> Self {
        ChannelId(CHANNEL_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for DotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dot({})", self.0)
    }
}

impl std::fmt::Display for CorpusId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Corpus({})", self.0)
    }
}
