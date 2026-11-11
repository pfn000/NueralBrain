//! Per-corpus runtime state enforcement.
//!
//! The controller gates pulse acceptance and per-frame processing. Corpus IDs
//! are stored dynamically because BioneticEngine supports custom corpus IDs.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use bionetic_core::{
    CorpusId, CorpusKind, RegionState, RegionStatus, ALL_CORPORA,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionTransition {
    WakeUp,
    GoToRest,
    GoDormant,
    BeginEnhancing,
    GoOffline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionEvent {
    pub corpus: CorpusId,
    pub from: RegionState,
    pub to: RegionState,
    pub tick: u64,
    pub reason: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
struct TickDecision {
    last_frame: Option<u64>,
    rest_counter: u32,
    allowed: bool,
}

pub struct RegionController {
    states: RwLock<HashMap<CorpusId, RegionStatus>>,
    tick_decisions: RwLock<HashMap<CorpusId, TickDecision>>,
    events: RwLock<VecDeque<RegionEvent>>,
    event_capacity: usize,
    fallback_frame: AtomicU64,
}

impl RegionController {
    /// Build the canonical 14-corpus layout; Auditory and SpeechMotor start Dormant.
    pub fn new_default() -> std::sync::Arc<Self> {
        Self::new_for_corpora(ALL_CORPORA)
    }

    /// Build a controller for the engine's actual corpus IDs.
    pub fn new_for_corpora<I>(corpora: I) -> std::sync::Arc<Self>
    where
        I: IntoIterator<Item = CorpusId>,
    {
        let mut states = HashMap::new();
        for corpus in corpora {
            let reserved = CorpusKind::from_id(corpus)
                .map(CorpusKind::is_reserved)
                .unwrap_or(false);
            states.insert(corpus, RegionStatus {
                corpus,
                state: if reserved { RegionState::Dormant } else { RegionState::Active },
                last_transition_tick: 0,
            });
        }
        std::sync::Arc::new(Self {
            states: RwLock::new(states),
            tick_decisions: RwLock::new(HashMap::new()),
            events: RwLock::new(VecDeque::with_capacity(256)),
            event_capacity: 256,
            fallback_frame: AtomicU64::new(0),
        })
    }

    /// Add a corpus that is created after controller initialisation.
    pub fn register(&self, corpus: CorpusId, state: RegionState) {
        self.states.write().entry(corpus).or_insert(RegionStatus {
            corpus,
            state,
            last_transition_tick: 0,
        });
    }

    /// Remove state for corpora that are no longer present in the engine.
    pub fn retain_corpora(&self, valid: &HashSet<CorpusId>) {
        self.states.write().retain(|id, _| valid.contains(id));
        self.tick_decisions.write().retain(|id, _| valid.contains(id));
    }

    /// Return the current status; unknown custom IDs are considered Active.
    pub fn status(&self, corpus: CorpusId) -> RegionStatus {
        self.states.read().get(&corpus).copied().unwrap_or(RegionStatus {
            corpus,
            state: RegionState::Active,
            last_transition_tick: 0,
        })
    }

    pub fn state(&self, corpus: CorpusId) -> RegionState {
        self.status(corpus).state
    }

    pub fn all_statuses(&self) -> Vec<RegionStatus> {
        let mut statuses: Vec<_> = self.states.read().values().copied().collect();
        statuses.sort_by_key(|status| status.corpus.0);
        statuses
    }

    /// External pulses are accepted in Active, Resting, or Dormant.
    pub fn should_deliver(&self, corpus: CorpusId) -> bool {
        self.state(corpus).accepts_external_pulses()
    }

    /// Advance the per-corpus tick decision once for a given engine frame.
    /// Repeated calls in the same frame return the same result.
    pub fn should_tick_this_frame_at(&self, corpus: CorpusId, frame: u64) -> bool {
        let state = self.state(corpus);
        let mut decisions = self.tick_decisions.write();
        let entry = decisions.entry(corpus).or_default();
        if entry.last_frame == Some(frame) {
            return entry.allowed;
        }
        entry.last_frame = Some(frame);
        entry.allowed = match state {
            RegionState::Active | RegionState::Enhancing => true,
            RegionState::Resting => {
                entry.rest_counter = entry.rest_counter.wrapping_add(1);
                entry.rest_counter % 10 == 0
            }
            RegionState::Dormant | RegionState::Offline => false,
        };
        entry.allowed
    }

    /// Compatibility helper for callers without an engine-frame counter.
    pub fn should_tick_this_frame(&self, corpus: CorpusId) -> bool {
        let frame = self.fallback_frame.fetch_add(1, Ordering::Relaxed) + 1;
        self.should_tick_this_frame_at(corpus, frame)
    }

    /// Transition a corpus. Repeating the current state is a no-op.
    pub fn transition(
        &self,
        corpus: CorpusId,
        to: RegionState,
        tick: u64,
        reason: &'static str,
    ) -> Option<RegionEvent> {
        let mut states = self.states.write();
        let status = states.entry(corpus).or_insert(RegionStatus {
            corpus,
            state: RegionState::Active,
            last_transition_tick: 0,
        });
        let from = status.state;
        if from == to {
            return None;
        }
        status.state = to;
        status.last_transition_tick = tick;
        let event = RegionEvent { corpus, from, to, tick, reason };
        let mut events = self.events.write();
        if events.len() == self.event_capacity {
            events.pop_front();
        }
        events.push_back(event.clone());
        Some(event)
    }

    pub fn transition_many<I>(
        &self,
        corpora: I,
        to: RegionState,
        tick: u64,
        reason: &'static str,
    ) -> Vec<RegionEvent>
    where
        I: IntoIterator<Item = CorpusId>,
    {
        corpora.into_iter()
            .filter_map(|corpus| self.transition(corpus, to, tick, reason))
            .collect()
    }

    pub fn recent_events(&self, n: usize) -> Vec<RegionEvent> {
        self.events.read().iter().rev().take(n).cloned().collect()
    }

    /// Counts in Active, Resting, Dormant, Enhancing, Offline order.
    pub fn census(&self) -> [usize; 5] {
        let mut counts = [0usize; 5];
        for status in self.states.read().values() {
            let index = match status.state {
                RegionState::Active => 0,
                RegionState::Resting => 1,
                RegionState::Dormant => 2,
                RegionState::Enhancing => 3,
                RegionState::Offline => 4,
            };
            counts[index] += 1;
        }
        counts
    }
}
