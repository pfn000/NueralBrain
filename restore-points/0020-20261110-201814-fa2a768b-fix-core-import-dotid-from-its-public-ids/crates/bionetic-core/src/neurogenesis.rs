//! # Neurogenesis — Adaptive Dot Creation & Pruning
//!
//! Biological brains grow new neurons in response to sustained demand
//! (hippocampal neurogenesis) and cull unused ones (synaptic pruning).
//! This module replicates both processes at the dot level.
//!
//! Growth is driven by corpus pulse rate. Pruning is driven by each
//! dot's firing rate over consecutive evaluation windows.
//!
//! Newborn dots start with empty connections and earn synapses through
//! the engine's existing Hebbian plasticity path.

use crate::dot::Dot;
use crate::ids::{CorpusId, DotId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tuning knobs for the neurogenesis controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurogenesisConfig {
    /// Activity rate (pulses / dot / tick) above which growth triggers.
    pub growth_threshold: f32,
    /// Consecutive evaluations above threshold before spawning.
    pub growth_patience: u32,
    /// How many new dots to spawn per trigger event.
    pub spawn_batch: usize,
    /// Hard cap on dots per corpus (0 = unlimited).
    pub max_dots_per_corpus: usize,

    /// Evaluation-window activity rate below which a dot is a prune candidate.
    pub prune_threshold: f32,
    /// Consecutive evaluations below threshold before pruning.
    pub prune_patience: u32,
    /// Ticks after birth during which a dot is immune to pruning.
    pub immunity_ticks: u64,

    /// Ticks between neurogenesis evaluation passes.
    pub eval_interval: u64,

    /// Resting membrane potential for newborn dots (mV).
    pub newborn_resting_mv: f32,
    /// Firing threshold for newborn dots (mV).
    pub newborn_threshold_mv: f32,
    /// Refractory period for newborn dots (ticks).
    pub newborn_refractory: u32,
}

impl Default for NeurogenesisConfig {
    fn default() -> Self {
        Self {
            growth_threshold: 0.6,
            growth_patience: 3,
            spawn_batch: 4,
            max_dots_per_corpus: 4096,
            prune_threshold: 0.01,
            prune_patience: 5,
            immunity_ticks: 2000,
            eval_interval: 500,
            newborn_resting_mv: -70.0,
            newborn_threshold_mv: -55.0,
            newborn_refractory: 2,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct CorpusStats {
    /// Total pulses observed since last evaluation.
    pulse_count: u64,
    /// Current dot count in this corpus.
    dot_count: usize,
    /// Ticks observed since last evaluation.
    ticks_observed: u64,
    /// Consecutive evaluations above growth threshold.
    growth_streak: u32,
}

impl CorpusStats {
    /// Pulses per dot per tick — the key growth signal.
    fn activity_rate(&self) -> f32 {
        if self.ticks_observed == 0 || self.dot_count == 0 {
            return 0.0;
        }

        self.pulse_count as f32 / (self.ticks_observed as f32 * self.dot_count as f32)
    }

    fn reset_window(&mut self) {
        self.pulse_count = 0;
        self.ticks_observed = 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotTracker {
    birth_tick: u64,
    lifetime_pulses: u64,
    window_pulses: u64,
    low_activity_streak: u32,
}

impl DotTracker {
    fn new(birth_tick: u64) -> Self {
        Self {
            birth_tick,
            lifetime_pulses: 0,
            window_pulses: 0,
            low_activity_streak: 0,
        }
    }

    fn activity_rate(&self, window_ticks: u64) -> f32 {
        if window_ticks == 0 {
            return 0.0;
        }

        self.window_pulses as f32 / window_ticks as f32
    }

    fn is_immune(&self, current_tick: u64, immunity_ticks: u64) -> bool {
        current_tick.saturating_sub(self.birth_tick) < immunity_ticks
    }

    fn reset_window(&mut self) {
        self.window_pulses = 0;
    }
}

/// Events emitted by the controller for observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NeuroEvent {
    Growth {
        corpus: CorpusId,
        count: usize,
        tick: u64,
    },
    Pruning {
        corpus: CorpusId,
        pruned_ids: Vec<DotId>,
        tick: u64,
    },
}

/// Adaptive dot growth and pruning controller.
#[derive(Debug, Serialize, Deserialize)]
pub struct NeurogenesisController {
    config: NeurogenesisConfig,
    corpus_stats: HashMap<CorpusId, CorpusStats>,
    dot_trackers: HashMap<DotId, DotTracker>,
    /// Required to route apoptosis events to the correct corpus.
    dot_corpora: HashMap<DotId, CorpusId>,
    tick_counter: u64,
    event_log: Vec<NeuroEvent>,
}

impl NeurogenesisController {
    /// Register pre-existing dots so they are not pruned on the first evaluation.
    pub fn new(config: NeurogenesisConfig, seed_dots: &[(DotId, u64)]) -> Self {
        let mut dot_trackers = HashMap::new();
        for &(id, birth_tick) in seed_dots {
            dot_trackers.insert(id, DotTracker::new(birth_tick));
        }

        Self {
            config,
            corpus_stats: HashMap::new(),
            dot_trackers,
            dot_corpora: HashMap::new(),
            tick_counter: 0,
            event_log: Vec::new(),
        }
    }

    /// Create with default config and no seed dots.
    pub fn empty() -> Self {
        Self::new(NeurogenesisConfig::default(), &[])
    }

    /// Record that a pulse arrived in a corpus.
    pub fn record_pulse(&mut self, corpus: CorpusId) {
        let stats = self.corpus_stats.entry(corpus).or_default();
        stats.pulse_count = stats.pulse_count.saturating_add(1);
    }

    /// Record that a specific dot fired this tick.
    pub fn record_dot_fire(&mut self, dot_id: DotId) {
        if let Some(tracker) = self.dot_trackers.get_mut(&dot_id) {
            tracker.lifetime_pulses = tracker.lifetime_pulses.saturating_add(1);
            tracker.window_pulses = tracker.window_pulses.saturating_add(1);
        }
    }

    /// Update the dot count for a corpus.
    pub fn set_dot_count(&mut self, corpus: CorpusId, count: usize) {
        self.corpus_stats.entry(corpus).or_default().dot_count = count;
    }

    /// Register a new dot or restore a dot from saved state.
    pub fn register_dot(&mut self, dot_id: DotId, birth_tick: u64) {
        self.dot_trackers.insert(dot_id, DotTracker::new(birth_tick));
    }

    /// Associate a tracked dot with its owning corpus.
    pub fn set_dot_corpus(&mut self, dot_id: DotId, corpus: CorpusId) {
        if self.dot_trackers.contains_key(&dot_id) {
            self.dot_corpora.insert(dot_id, corpus);
        }
    }

    /// Register a dot and its owning corpus in one operation.
    pub fn register_dot_in_corpus(
        &mut self,
        dot_id: DotId,
        corpus: CorpusId,
        birth_tick: u64,
    ) {
        self.register_dot(dot_id, birth_tick);
        self.set_dot_corpus(dot_id, corpus);
    }

    /// Unregister a dot after it is removed from the engine.
    pub fn unregister_dot(&mut self, dot_id: &DotId) {
        self.dot_trackers.remove(dot_id);
        self.dot_corpora.remove(dot_id);
    }

    /// Advance the clock by one tick and return events for the engine to apply.
    #[allow(clippy::manual_is_multiple_of)]
    pub fn tick(&mut self, current_tick: u64) -> Vec<NeuroEvent> {
        self.tick_counter = current_tick;

        for stats in self.corpus_stats.values_mut() {
            stats.ticks_observed = stats.ticks_observed.saturating_add(1);
        }

        // Treat zero as one to avoid modulo-by-zero after runtime tuning.
        let eval_interval = self.config.eval_interval.max(1);
        if current_tick % eval_interval != 0 {
            return Vec::new();
        }

        self.evaluate(current_tick)
    }

    fn evaluate(&mut self, current_tick: u64) -> Vec<NeuroEvent> {
        let mut events = self.evaluate_growth(current_tick);
        events.extend(self.evaluate_pruning(current_tick));

        for stats in self.corpus_stats.values_mut() {
            stats.reset_window();
        }
        for tracker in self.dot_trackers.values_mut() {
            tracker.reset_window();
        }

        self.event_log.extend(events.iter().cloned());
        events
    }

    fn evaluate_growth(&mut self, current_tick: u64) -> Vec<NeuroEvent> {
        let mut events = Vec::new();
        let config = self.config.clone();
        let corpus_ids: Vec<CorpusId> = self.corpus_stats.keys().copied().collect();

        for corpus in corpus_ids {
            let stats = self.corpus_stats.get_mut(&corpus).expect("corpus stats exist");
            if stats.activity_rate() > config.growth_threshold {
                stats.growth_streak = stats.growth_streak.saturating_add(1);
            } else {
                stats.growth_streak = 0;
            }

            if stats.growth_streak < config.growth_patience {
                continue;
            }

            let room = if config.max_dots_per_corpus == 0 {
                config.spawn_batch
            } else {
                config.max_dots_per_corpus.saturating_sub(stats.dot_count)
            };
            let count = room.min(config.spawn_batch);

            if count == 0 {
                stats.growth_streak = 0;
                continue;
            }

            // Reserve capacity; the engine inserts the actual newborn Dot values
            // after consuming this event, then refreshes counts on its next tick.
            stats.dot_count = stats.dot_count.saturating_add(count);
            stats.growth_streak = 0;
            events.push(NeuroEvent::Growth {
                corpus,
                count,
                tick: current_tick,
            });
        }

        events
    }

    fn evaluate_pruning(&mut self, current_tick: u64) -> Vec<NeuroEvent> {
        let config = self.config.clone();
        let window_ticks = self.config.eval_interval.max(1);
        let mut candidates: HashMap<CorpusId, Vec<DotId>> = HashMap::new();

        for (&dot_id, tracker) in &mut self.dot_trackers {
            if tracker.is_immune(current_tick, config.immunity_ticks) {
                continue;
            }

            if tracker.activity_rate(window_ticks) < config.prune_threshold {
                tracker.low_activity_streak =
                    tracker.low_activity_streak.saturating_add(1);
            } else {
                tracker.low_activity_streak = 0;
            }

            if tracker.low_activity_streak < config.prune_patience {
                continue;
            }

            if let Some(&corpus) = self.dot_corpora.get(&dot_id) {
                candidates.entry(corpus).or_default().push(dot_id);
            }
        }

        let mut events = Vec::new();
        for (corpus, ids) in candidates {
            if ids.is_empty() {
                continue;
            }

            for id in &ids {
                self.dot_trackers.remove(id);
                self.dot_corpora.remove(id);
            }

            events.push(NeuroEvent::Pruning {
                corpus,
                pruned_ids: ids,
                tick: current_tick,
            });
        }

        events
    }

    /// Get the current activity rate for a corpus.
    pub fn corpus_activity_rate(&self, corpus: CorpusId) -> f32 {
        self.corpus_stats
            .get(&corpus)
            .map(CorpusStats::activity_rate)
            .unwrap_or(0.0)
    }

    /// Get lifetime firing count for a dot.
    pub fn dot_lifetime_pulses(&self, dot_id: &DotId) -> u64 {
        self.dot_trackers
            .get(dot_id)
            .map(|tracker| tracker.lifetime_pulses)
            .unwrap_or(0)
    }

    /// Total dots being tracked.
    pub fn tracked_dot_count(&self) -> usize {
        self.dot_trackers.len()
    }

    /// Current controller tick.
    pub fn current_tick(&self) -> u64 {
        self.tick_counter
    }

    /// Drain the event log for serialization or UI display.
    pub fn drain_events(&mut self) -> Vec<NeuroEvent> {
        self.event_log.drain(..).collect()
    }

    /// Access the config for runtime tuning.
    pub fn config(&self) -> &NeurogenesisConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut NeurogenesisConfig {
        &mut self.config
    }

    /// Generate actual Dot structs after the engine receives a Growth event.
    /// Newborns use Dot::new, so connections and chemistry start empty.
    pub fn create_newborn_dots(
        &mut self,
        corpus: CorpusId,
        count: usize,
        birth_tick: u64,
    ) -> Vec<Dot> {
        let mut dots = Vec::with_capacity(count);

        for _ in 0..count {
            // Use the engine's shared ID generator and skip manually-seeded IDs.
            let mut id = DotId::next();
            while self.dot_trackers.contains_key(&id) {
                id = DotId::next();
            }

            let mut dot = Dot::new(id, corpus, birth_tick);
            let resting = self.config.newborn_resting_mv as f64;
            let threshold = self.config.newborn_threshold_mv as f64;
            dot.potential = resting;
            dot.resting_potential = resting;
            dot.base_threshold = threshold;
            dot.threshold = threshold;
            dot.refractory_duration = self.config.newborn_refractory;

            self.register_dot_in_corpus(id, corpus, birth_tick);
            dots.push(dot);
        }

        dots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> NeurogenesisConfig {
        NeurogenesisConfig {
            growth_threshold: 0.5,
            growth_patience: 2,
            spawn_batch: 3,
            max_dots_per_corpus: 100,
            prune_threshold: 0.01,
            prune_patience: 2,
            immunity_ticks: 100,
            eval_interval: 10,
            newborn_resting_mv: -70.0,
            newborn_threshold_mv: -55.0,
            newborn_refractory: 2,
        }
    }

    #[test]
    fn test_no_growth_below_threshold() {
        let mut neuro = NeurogenesisController::new(test_config(), &[]);
        neuro.set_dot_count(CorpusId(0), 10);

        for tick in 1..=30 {
            neuro.record_pulse(CorpusId(0));
            assert!(
                neuro.tick(tick).is_empty(),
                "Should not grow at tick {tick} with low activity"
            );
        }
    }

    #[test]
    fn test_growth_triggers_after_patience() {
        let mut neuro = NeurogenesisController::new(test_config(), &[]);
        neuro.set_dot_count(CorpusId(0), 2);
        let mut growth_seen = false;

        for tick in 1..=30 {
            for _ in 0..5 {
                neuro.record_pulse(CorpusId(0));
            }

            for event in neuro.tick(tick) {
                if let NeuroEvent::Growth { corpus, count, .. } = event {
                    assert_eq!(corpus, CorpusId(0));
                    assert_eq!(count, 3);
                    growth_seen = true;
                }
            }
        }

        assert!(growth_seen, "Growth should have triggered");
    }

    #[test]
    fn test_growth_respects_max_cap() {
        let mut config = test_config();
        config.max_dots_per_corpus = 5;
        let mut neuro = NeurogenesisController::new(config, &[]);
        neuro.set_dot_count(CorpusId(0), 4);

        for tick in 1..=30 {
            for _ in 0..10 {
                neuro.record_pulse(CorpusId(0));
            }

            for event in neuro.tick(tick) {
                if let NeuroEvent::Growth { count, .. } = event {
                    assert!(count <= 1, "Should not exceed cap");
                }
            }
        }
    }

    #[test]
    fn test_newborn_dots_have_correct_params() {
        let mut neuro = NeurogenesisController::new(test_config(), &[]);
        let dots = neuro.create_newborn_dots(CorpusId(1), 2, 100);

        assert_eq!(dots.len(), 2);
        for dot in &dots {
            assert_eq!(dot.corpus, CorpusId(1));
            assert_eq!(dot.resting_potential, -70.0);
            assert_eq!(dot.threshold, -55.0);
            assert_eq!(dot.refractory_duration, 2);
            assert!(dot.endings.is_empty());
            assert!(dot.branches.is_empty());
        }
        assert_eq!(neuro.tracked_dot_count(), 2);
    }

    #[test]
    fn test_dot_tracking() {
        let seed = vec![(DotId(1), 0), (DotId(2), 0)];
        let mut neuro = NeurogenesisController::new(test_config(), &seed);

        neuro.record_dot_fire(DotId(1));
        neuro.record_dot_fire(DotId(1));
        neuro.record_dot_fire(DotId(2));

        assert_eq!(neuro.dot_lifetime_pulses(&DotId(1)), 2);
        assert_eq!(neuro.dot_lifetime_pulses(&DotId(2)), 1);
        assert_eq!(neuro.dot_lifetime_pulses(&DotId(99)), 0);
    }

    #[test]
    fn test_immune_dots_not_pruned() {
        let seed = vec![(DotId(1), 5)];
        let mut neuro = NeurogenesisController::new(test_config(), &seed);
        neuro.set_dot_corpus(DotId(1), CorpusId(0));
        neuro.set_dot_count(CorpusId(0), 1);

        for tick in 1..=20 {
            for event in neuro.tick(tick) {
                if let NeuroEvent::Pruning { pruned_ids, .. } = event {
                    assert!(!pruned_ids.contains(&DotId(1)), "Immune dot should not be pruned");
                }
            }
        }
    }

    #[test]
    fn test_pruning_targets_the_correct_corpus() {
        let mut config = test_config();
        config.immunity_ticks = 0;
        let seed = vec![(DotId(1001), 0), (DotId(2001), 0)];
        let mut neuro = NeurogenesisController::new(config, &seed);
        neuro.set_dot_corpus(DotId(1001), CorpusId(0));
        neuro.set_dot_corpus(DotId(2001), CorpusId(1));

        for tick in 1..=20 {
            neuro.record_dot_fire(DotId(2001));
            for event in neuro.tick(tick) {
                if let NeuroEvent::Pruning { corpus, pruned_ids, .. } = event {
                    assert_eq!(corpus, CorpusId(0));
                    assert_eq!(pruned_ids, vec![DotId(1001)]);
                }
            }
        }

        assert_eq!(neuro.tracked_dot_count(), 1);
        assert_eq!(neuro.dot_lifetime_pulses(&DotId(2001)), 20);
    }

    #[test]
    fn test_activity_rate_calculation() {
        let mut stats = CorpusStats::default();
        stats.pulse_count = 100;
        stats.dot_count = 10;
        stats.ticks_observed = 50;
        assert!((stats.activity_rate() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_empty_controller() {
        let neuro = NeurogenesisController::empty();
        assert_eq!(neuro.tracked_dot_count(), 0);
        assert_eq!(neuro.corpus_activity_rate(CorpusId(0)), 0.0);
    }
}
