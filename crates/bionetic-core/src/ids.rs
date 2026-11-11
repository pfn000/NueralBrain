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


/// Canonical corpus identifiers. Prefer these names over numeric IDs.
pub const SENSORY_IN: CorpusId = CorpusId(0);
pub const SENSORY_OUT: CorpusId = CorpusId(1);
pub const VISION: CorpusId = CorpusId(2);
pub const PREFRONTAL: CorpusId = CorpusId(3);
pub const ASSOCIATION: CorpusId = CorpusId(4);
pub const RELATIONAL_CONTEXT: CorpusId = CorpusId(5);
pub const SHORT_TERM_MEMORY: CorpusId = CorpusId(6);
pub const HIPPOCAMPUS: CorpusId = CorpusId(7);
pub const LONG_TERM_MEMORY: CorpusId = CorpusId(8);
pub const REWARD: CorpusId = CorpusId(9);
pub const MOTOR: CorpusId = CorpusId(10);
pub const DREAM: CorpusId = CorpusId(11);
pub const AUDITORY: CorpusId = CorpusId(12);
pub const SPEECH_MOTOR: CorpusId = CorpusId(13);

pub const CORPUS_COUNT_ACTIVE: usize = 12;
pub const CORPUS_COUNT_RESERVED: usize = 2;
pub const CORPUS_COUNT_TOTAL: usize = CORPUS_COUNT_ACTIVE + CORPUS_COUNT_RESERVED;

pub const ALL_CORPORA: [CorpusId; CORPUS_COUNT_TOTAL] = [
    SENSORY_IN, SENSORY_OUT, VISION, PREFRONTAL, ASSOCIATION,
    RELATIONAL_CONTEXT, SHORT_TERM_MEMORY, HIPPOCAMPUS, LONG_TERM_MEMORY,
    REWARD, MOTOR, DREAM, AUDITORY, SPEECH_MOTOR,
];

/// A logical slot shared across corpus regions.
///
/// DotIds in this engine are globally unique u64 values, not (corpus, index)
/// pairs. SpanDotId therefore stores logical per-corpus slots and resolves them
/// through an engine-provided lookup without changing existing DotId identity.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanDotId {
    pub corpora: Vec<CorpusId>,
    pub index: usize,
}

impl SpanDotId {
    pub fn new(corpora: Vec<CorpusId>, index: usize) -> Self {
        debug_assert!(!corpora.is_empty(), "SpanDotId must span at least one corpus");
        Self { corpora, index }
    }

    pub fn single(corpus: CorpusId, index: usize) -> Self {
        Self { corpora: vec![corpus], index }
    }

    pub fn as_slots(&self) -> impl Iterator<Item = (CorpusId, usize)> + '_ {
        self.corpora.iter().copied().map(move |corpus| (corpus, self.index))
    }

    pub fn resolve<F>(&self, mut lookup: F) -> Vec<DotId>
    where
        F: FnMut(CorpusId, usize) -> Option<DotId>,
    {
        self.as_slots().filter_map(|(corpus, slot)| lookup(corpus, slot)).collect()
    }

    pub fn spans(&self, corpus: CorpusId) -> bool { self.corpora.contains(&corpus) }
    pub fn len(&self) -> usize { self.corpora.len() }
    pub fn is_empty(&self) -> bool { self.corpora.is_empty() }
}

impl fmt::Display for SpanDotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "span[")?;
        for (i, corpus) in self.corpora.iter().enumerate() {
            if i > 0 { write!(f, ",")?; }
            write!(f, "{}", corpus.0)?;
        }
        write!(f, "]#{}", self.index)
    }
}

pub fn possession_dot(index: usize) -> SpanDotId {
    SpanDotId::new(
        vec![SENSORY_IN, SENSORY_OUT, RELATIONAL_CONTEXT, SHORT_TERM_MEMORY, MOTOR],
        index,
    )
}

impl std::fmt::Display for CorpusId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Corpus({})", self.0)
    }
}
