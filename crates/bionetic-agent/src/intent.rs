use bionetic_core::ids::{CorpusId, DotId};
use bionetic_core::MOTOR;
use bionetic_engine::engine::BioneticEngine;

use crate::{INTENT_FIRE_THRESHOLD, INTENT_WINDOW_TICKS};

/// High-level categories represented by motor-corpus regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IntentKind {
    Idle,
    Query,
    Act,
    Navigate,
    Emit,
    Reflect,
    Custom(u16),
}

/// A decoded motor intent and its raw activation payload.
#[derive(Debug, Clone)]
pub struct Intent {
    pub kind: IntentKind,
    pub strength: f32,
    pub payload: Vec<u8>,
    pub tick: u64,
}

impl Intent {
    pub fn idle(tick: u64) -> Self {
        Self { kind: IntentKind::Idle, strength: 0.0, payload: Vec::new(), tick }
    }
}

/// Motor-corpus region layout. Offsets address dots in stable DotId order.
pub mod motor_layout {
    pub const QUERY_OFFSET: usize = 0;
    pub const QUERY_SIZE: usize = 16;
    pub const ACT_OFFSET: usize = 16;
    pub const ACT_SIZE: usize = 16;
    pub const NAVIGATE_OFFSET: usize = 32;
    pub const NAVIGATE_SIZE: usize = 16;
    pub const EMIT_OFFSET: usize = 48;
    pub const EMIT_SIZE: usize = 16;
    pub const REFLECT_OFFSET: usize = 64;
    pub const REFLECT_SIZE: usize = 16;
    pub const PAYLOAD_OFFSET: usize = 80;
    pub const PAYLOAD_SIZE: usize = 48;
}

pub struct IntentDecoder {
    motor_corpus: CorpusId,
    threshold: f32,
    window_ticks: u64,
}

impl Default for IntentDecoder {
    fn default() -> Self {
        Self::new(MOTOR, INTENT_FIRE_THRESHOLD)
    }
}

impl IntentDecoder {
    pub fn new(motor_corpus: CorpusId, threshold: f32) -> Self {
        Self { motor_corpus, threshold: threshold.clamp(0.0, 1.0), window_ticks: INTENT_WINDOW_TICKS }
    }

    pub fn with_window(mut self, ticks: u64) -> Self {
        self.window_ticks = ticks.max(1);
        self
    }

    /// Decode the fraction of recently firing dots in each motor region.
    pub fn decode(&self, engine: &BioneticEngine) -> Intent {
        use motor_layout::*;

        let tick = engine.tick;
        let Some(corpus) = engine.corpora.get(&self.motor_corpus) else {
            return Intent::idle(tick);
        };

        let mut dots: Vec<_> = corpus.dots.values().collect();
        dots.sort_by_key(|dot| dot.id.0);

        let region_rate = |offset: usize, size: usize| -> f32 {
            if offset >= dots.len() { return 0.0; }
            let end = offset.saturating_add(size).min(dots.len());
            let slice = &dots[offset..end];
            if slice.is_empty() { return 0.0; }
            let active = slice.iter()
                .filter(|dot| dot.fired_recently(tick, self.window_ticks))
                .count();
            active as f32 / slice.len() as f32
        };

        let candidates = [
            (IntentKind::Query, region_rate(QUERY_OFFSET, QUERY_SIZE)),
            (IntentKind::Act, region_rate(ACT_OFFSET, ACT_SIZE)),
            (IntentKind::Navigate, region_rate(NAVIGATE_OFFSET, NAVIGATE_SIZE)),
            (IntentKind::Emit, region_rate(EMIT_OFFSET, EMIT_SIZE)),
            (IntentKind::Reflect, region_rate(REFLECT_OFFSET, REFLECT_SIZE)),
        ];

        let (kind, strength) = candidates.iter()
            .copied()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((IntentKind::Idle, 0.0));

        if strength < self.threshold {
            return Intent::idle(tick);
        }

        let payload = dots.iter()
            .skip(PAYLOAD_OFFSET)
            .take(PAYLOAD_SIZE)
            .map(|dot| {
                if dot.fired_recently(tick, self.window_ticks) { u8::MAX } else { 0 }
            })
            .collect();

        Intent { kind, strength, payload, tick }
    }

    /// Resolve an ordinal motor offset to its actual engine DotId.
    pub fn motor_dot(&self, engine: &BioneticEngine, offset: usize) -> Option<DotId> {
        let corpus = engine.corpora.get(&self.motor_corpus)?;
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        ids.get(offset).copied()
    }
}
