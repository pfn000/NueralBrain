use std::collections::{HashMap, VecDeque};
use bionetic_core::*;
use crate::hippocampus::HippocampalEngine;
use crate::reward::RewardSystem;
use crate::dream::DreamEngine;
use bionetic_core::neurogenesis::{NeuroEvent, NeurogenesisController};

/// A pulse scheduled for delivery
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PulseDelivery {
    pub pulse: Pulse,
    pub target_dot: DotId,
    pub target_corpus: CorpusId,
    pub target_gate: GateId,
    pub delivery_tick: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SystemState {
    Awake,
    Drowsy,
    Dreaming,
    DeepSleep,
    Booting,
}

pub struct BioneticEngine {
    pub corpora: HashMap<CorpusId, Corpus>,
    pub trans_channels: Vec<TransBinaryChannel>,
    pub pulse_queue: VecDeque<PulseDelivery>,
    pub tick: u64,
    pub state: SystemState,
    pub dream_engine: DreamEngine,
    pub hippocampus: HippocampalEngine,
    pub reward_system: RewardSystem,
    neurogenesis: NeurogenesisController,
    /// Maps logical modality addresses to physical sensory-corpus dots.
    multimodal_dot_map: HashMap<DotId, DotId>,
    neurogenesis_enabled: bool,
    last_propagated_pulses: usize,
}

impl BioneticEngine {
    pub fn new() -> Self {
        let mut engine = BioneticEngine {
            corpora: HashMap::new(),
            trans_channels: Vec::new(),
            pulse_queue: VecDeque::new(),
            tick: 0,
            state: SystemState::Booting,
            dream_engine: DreamEngine::new(),
            hippocampus: HippocampalEngine::new(),
            reward_system: RewardSystem::new(),
            neurogenesis: NeurogenesisController::empty(),
            multimodal_dot_map: HashMap::new(),
            neurogenesis_enabled: true,
            last_propagated_pulses: 0,
        };

        engine.initialize();
        engine.state = SystemState::Awake;
        engine
    }

    fn initialize(&mut self) {
        // Create corpora
        let regions = vec![
            (CorpusId(0), "Sensory", CorpusType::Sensory),
            (CorpusId(1), "Prefrontal", CorpusType::Prefrontal),
            (CorpusId(2), "Association", CorpusType::Association),
            (CorpusId(3), "Hippocampus", CorpusType::Hippocampus),
            (CorpusId(4), "Reward", CorpusType::Reward),
            (CorpusId(5), "Motor", CorpusType::Motor),
            (CorpusId(6), "LongTermMemory", CorpusType::LongTermMemory),
            (CorpusId(7), "Dream", CorpusType::Dream),
        ];

        for (id, name, ctype) in regions {
            self.corpora.insert(id, Corpus::new(id, name, ctype));
        }

        // Seed dots
        for corpus_id in 0..8u32 {
            for _ in 0..64 {
                self.spawn_dot(CorpusId(corpus_id));
            }
        }

        // Create random intra-corpus connections
        for corpus_id in 0..8u32 {
            let cid = CorpusId(corpus_id);
            let dot_ids: Vec<DotId> = self.corpora[&cid].dots.keys().cloned().collect();
            let len = dot_ids.len();

            // Each dot gets 3-6 random connections within its corpus
            for i in 0..len {
                let num_connections = 3 + (i % 4);
                for j in 1..=num_connections {
                    let target_idx = (i + j * 7 + j * j) % len;
                    if target_idx != i {
                        let source = dot_ids[i];
                        let target = dot_ids[target_idx];
                        let gate = Gate::soft();
                        let gate_id = gate.id;

                        if let Some(corpus) = self.corpora.get_mut(&cid) {
                            corpus.gates.insert(gate_id, gate);
                            if let Some(dot) = corpus.dots.get_mut(&source) {
                                dot.add_branch(target, cid, gate_id, 0.3 + (i as f64 * 0.01) % 0.7);
                            }
                            if let Some(dot) = corpus.dots.get_mut(&target) {
                                dot.add_ending(source, gate_id, 0.3 + (i as f64 * 0.01) % 0.7);
                            }
                        }
                    }
                }
            }
        }

        // Trans-binary channels
        let channel_pairs = vec![
            (CorpusId(0), CorpusId(2)), // Sensory ↔ Association
            (CorpusId(2), CorpusId(1)), // Association ↔ Prefrontal
            (CorpusId(2), CorpusId(3)), // Association ↔ Hippocampus
            (CorpusId(2), CorpusId(4)), // Association ↔ Reward
            (CorpusId(1), CorpusId(5)), // Prefrontal ↔ Motor
            (CorpusId(3), CorpusId(6)), // Hippocampus ↔ LTM
            (CorpusId(7), CorpusId(3)), // Dream ↔ Hippocampus
            (CorpusId(7), CorpusId(2)), // Dream ↔ Association
        ];

        for (a, b) in channel_pairs {
            self.trans_channels.push(TransBinaryChannel::new(a, b));
        }
    }

    pub fn spawn_dot(&mut self, corpus_id: CorpusId) -> DotId {
        let id = DotId::next();
        let dot = Dot::new(id, corpus_id, self.tick);

        let inserted = if let Some(corpus) = self.corpora.get_mut(&corpus_id) {
            corpus.dots.insert(id, dot);
            true
        } else {
            false
        };

        if inserted {
            self.neurogenesis
                .register_dot_in_corpus(id, corpus_id, self.tick);
        }

        id
    }

    pub fn tick(&mut self) {
        self.tick += 1;
        self.last_propagated_pulses = 0;

        match self.state {
            SystemState::Awake => self.tick_awake(),
            SystemState::Drowsy => self.tick_drowsy(),
            SystemState::Dreaming => self.tick_dreaming(),
            SystemState::DeepSleep => self.tick_deep_sleep(),
            SystemState::Booting => {}
        }

        if self.neurogenesis_enabled {
            self.tick_neurogenesis();
        }
    }

    // Keep modulo checks compatible with the documented Rust 1.75+ floor.
    #[allow(unknown_lints, clippy::manual_is_multiple_of)]
    fn tick_awake(&mut self) {
        self.deliver_pulses();

        let corpus_ids: Vec<CorpusId> = self.corpora.keys().cloned().collect();
        for cid in &corpus_ids {
            self.process_corpus(*cid);
        }

        self.manage_clumps();
        self.update_chemicals();

        if self.tick % 10 == 0 {
            self.apply_plasticity();
        }

        if self.tick % 100 == 0 {
            self.consolidate_memory();
        }

        self.update_fatigue();
        self.check_sleep_transition();
    }

    fn tick_drowsy(&mut self) {
        self.deliver_pulses();
        self.process_corpus(CorpusId(1));
        self.process_corpus(CorpusId(3));
        self.update_chemicals();
        self.update_fatigue();
    }

    fn tick_dreaming(&mut self) {
        self.dream_engine.dream_tick(&mut self.corpora, self.tick);

        // Recover fatigue during dreams
        for corpus in self.corpora.values_mut() {
            for dot in corpus.dots.values_mut() {
                dot.recover_fatigue(0.005);
            }
        }

        // Check if should wake
        let avg_fatigue = self.average_fatigue();
        if avg_fatigue < 0.1 {
            self.state = SystemState::Awake;
        }
    }

    fn tick_deep_sleep(&mut self) {
        self.consolidate_memory();

        for corpus in self.corpora.values_mut() {
            for dot in corpus.dots.values_mut() {
                dot.recover_fatigue(0.01);
            }
        }

        let avg_fatigue = self.average_fatigue();
        if avg_fatigue < 0.2 {
            self.state = SystemState::Dreaming;
        }
    }

    fn deliver_pulses(&mut self) {
        let mut remaining = VecDeque::new();
        let mut delivered_corpora = Vec::new();

        while let Some(delivery) = self.pulse_queue.pop_front() {
            if delivery.delivery_tick <= self.tick {
                if let Some(corpus) = self.corpora.get_mut(&delivery.target_corpus) {
                    if let Some(dot) = corpus.dots.get_mut(&delivery.target_dot) {
                        dot.potential += delivery.pulse.value;
                        dot.chemicals.apply_delta(&delivery.pulse.chemical_payload);
                        delivered_corpora.push(delivery.target_corpus);
                    }
                }
            } else {
                remaining.push_back(delivery);
            }
        }

        self.pulse_queue = remaining;

        for corpus_id in delivered_corpora {
            self.neurogenesis.record_pulse(corpus_id);
        }
    }

    fn process_corpus(&mut self, corpus_id: CorpusId) {
        let mut new_deliveries: Vec<PulseDelivery> = Vec::new();
        let mut fired_dots = Vec::new();

        if let Some(corpus) = self.corpora.get_mut(&corpus_id) {
            corpus.cycle_count += 1;
            let tick = self.tick;

            let dot_ids: Vec<DotId> = corpus.dots.keys().cloned().collect();

            for dot_id in dot_ids {
                if let Some(dot) = corpus.dots.get_mut(&dot_id) {
                    dot.tick_refractory();
                    dot.update_threshold();

                    if dot.should_fire() {
                        let pulse_value = dot.potential - dot.resting_potential;
                        dot.fire(tick);
                        fired_dots.push(dot_id);

                        // Generate pulses for each branch
                        for branch in &mut dot.branches {
                            let pulse = Pulse::value_pulse(
                                dot_id,
                                pulse_value * branch.strength,
                                (pulse_value / 20.0).abs(),
                                tick,
                            );

                            branch.transmission_count += 1;

                            new_deliveries.push(PulseDelivery {
                                pulse,
                                target_dot: branch.target_dot,
                                target_corpus: branch.target_corpus,
                                target_gate: branch.gate,
                                delivery_tick: tick + branch.delay_ticks as u64,
                            });
                        }
                    } else {
                        dot.decay_potential(0.1);
                    }
                }
            }
        }

        self.last_propagated_pulses = self.last_propagated_pulses.saturating_add(new_deliveries.len());
        for delivery in new_deliveries {
            self.pulse_queue.push_back(delivery);
        }

        for dot_id in fired_dots {
            self.neurogenesis.record_dot_fire(dot_id);
        }
    }

    fn tick_neurogenesis(&mut self) {
        let corpus_counts: Vec<(CorpusId, usize)> = self
            .corpora
            .iter()
            .map(|(corpus_id, corpus)| (*corpus_id, corpus.dot_count()))
            .collect();

        for (corpus_id, count) in corpus_counts {
            self.neurogenesis.set_dot_count(corpus_id, count);
        }

        let events = self.neurogenesis.tick(self.tick);
        for event in events {
            match event {
                NeuroEvent::Growth { corpus, count, tick } => {
                    let newborns =
                        self.neurogenesis.create_newborn_dots(corpus, count, tick);

                    if let Some(target) = self.corpora.get_mut(&corpus) {
                        for dot in newborns {
                            target.dots.insert(dot.id, dot);
                        }
                    } else {
                        for dot in newborns {
                            self.neurogenesis.unregister_dot(&dot.id);
                        }
                    }
                }
                NeuroEvent::Pruning { corpus, pruned_ids, .. } => {
                    if let Some(target) = self.corpora.get_mut(&corpus) {
                        for dot_id in &pruned_ids {
                            target.dots.remove(dot_id);
                        }
                    }
                    for dot_id in &pruned_ids {
                        self.multimodal_dot_map
                            .retain(|_, mapped_dot| mapped_dot != dot_id);
                    }
                }
            }
        }
    }

    fn manage_clumps(&mut self) {
        let tick = self.tick;
        let synchrony_window = 5u64;
        let min_clump_size = 3usize;
        let max_clump_age = 3000u64;

        for corpus in self.corpora.values_mut() {
            // Find recently fired dots
            let recently_fired: Vec<DotId> = corpus.dots.values()
                .filter(|d| d.fired_recently(tick, synchrony_window))
                .map(|d| d.id)
                .collect();

            // Form clump if enough synchronous firing
            if recently_fired.len() >= min_clump_size {
                let clump_id = ClumpId::next();
                let clump = Clump::new(
                    clump_id,
                    recently_fired.clone(),
                    format!("sync_tick_{}", tick),
                    tick,
                );
                corpus.clumps.insert(clump_id, clump);

                for dot_id in &recently_fired {
                    if let Some(dot) = corpus.dots.get_mut(dot_id) {
                        dot.clump_id = Some(clump_id);
                    }
                }
            }

            // Clean stale clumps
            let stale: Vec<ClumpId> = corpus.clumps.iter()
                .filter(|(_, c)| c.is_stale(tick, max_clump_age))
                .map(|(id, _)| *id)
                .collect();

            for id in stale {
                corpus.clumps.remove(&id);
            }
        }
    }

    fn update_chemicals(&mut self) {
        for corpus in self.corpora.values_mut() {
            for dot in corpus.dots.values_mut() {
                dot.chemicals.decay(0.01);
            }
        }
    }

    fn apply_plasticity(&mut self) {
        let tick = self.tick;
        let hebbian_window = 20u64;

        for corpus in self.corpora.values_mut() {
            let dot_ids: Vec<DotId> = corpus.dots.keys().cloned().collect();

            for dot_id in &dot_ids {
                let (source_fired, branches) = {
                    if let Some(dot) = corpus.dots.get(dot_id) {
                        (dot.fired_recently(tick, hebbian_window), dot.branches.clone())
                    } else {
                        continue;
                    }
                };

                if source_fired {
                    for branch in &branches {
                        let target_fired = corpus.dots.get(&branch.target_dot)
                            .map(|d| d.fired_recently(tick, hebbian_window))
                            .unwrap_or(false);

                        if target_fired {
                            // Hebbian: strengthen
                            if let Some(dot) = corpus.dots.get_mut(dot_id) {
                                for b in &mut dot.branches {
                                    if b.target_dot == branch.target_dot {
                                        b.strength = (b.strength + 0.005).min(5.0);
                                        b.success_rate = (b.success_rate + 0.002).min(1.0);
                                    }
                                }
                            }
                        }
                    }
                }

                // Weaken unused
                if let Some(dot) = corpus.dots.get_mut(dot_id) {
                    for branch in &mut dot.branches {
                        if branch.transmission_count == 0 && tick > 1000 {
                            branch.strength = (branch.strength - 0.0005).max(0.01);
                        }
                    }
                }
            }
        }
    }

    fn consolidate_memory(&mut self) {
        self.hippocampus.consolidate(&mut self.corpora);
    }

    fn update_fatigue(&mut self) {
        for corpus in self.corpora.values_mut() {
            for dot in corpus.dots.values_mut() {
                dot.recover_fatigue(0.001);
            }
        }
    }

    fn check_sleep_transition(&mut self) {
        let avg = self.average_fatigue();
        if avg > 0.7 {
            self.state = SystemState::Drowsy;
        }
        if avg > 0.9 {
            self.state = SystemState::Dreaming;
        }
    }

    fn average_fatigue(&self) -> f64 {
        let (sum, count) = self.corpora.values()
            .flat_map(|c| c.dots.values())
            .fold((0.0, 0usize), |(s, c), d| (s + d.fatigue, c + 1));

        if count > 0 { sum / count as f64 } else { 0.0 }
    }

    // === PUBLIC API ===

    pub fn total_dots(&self) -> usize {
        self.corpora.values().map(|c| c.dot_count()).sum()
    }

    pub fn total_corpora(&self) -> usize {
        self.corpora.len()
    }

    pub fn total_connections(&self) -> usize {
        self.corpora.values().map(|c| c.connection_count()).sum()
    }

    pub fn total_clumps(&self) -> usize {
        self.corpora.values().map(|c| c.clumps.len()).sum()
    }

    pub fn pending_pulses(&self) -> usize {
        self.pulse_queue.len()
    }

    /// Access the neurogenesis controller for runtime inspection and tuning.
    pub fn neurogenesis(&self) -> &NeurogenesisController {
        &self.neurogenesis
    }

    pub fn neurogenesis_mut(&mut self) -> &mut NeurogenesisController {
        &mut self.neurogenesis
    }

    pub fn enter_sleep(&mut self) {
        self.state = SystemState::Dreaming;
    }

    pub fn wake_up(&mut self) {
        self.state = SystemState::Awake;
    }

    pub fn reward(&mut self, magnitude: f64, behavior: &str) {
        self.reward_system.reward(magnitude, behavior, &mut self.corpora, self.tick);
    }

    pub fn discipline(&mut self, magnitude: f64, behavior: &str) {
        self.reward_system.discipline(magnitude, behavior, &mut self.corpora, self.tick);
    }


    /// Construct an engine with a selected number of corpora and initial dots per corpus.
    pub fn with_config(num_corpora: usize, dots_per_corpus: usize, enable_neurogenesis: bool) -> Result<Self, String> {
        if num_corpora == 0 || num_corpora > 256 {
            return Err("num_corpora must be between 1 and 256".to_string());
        }
        if dots_per_corpus == 0 || dots_per_corpus > 4096 {
            return Err("dots_per_corpus must be between 1 and 4096".to_string());
        }
        let mut engine = Self::new();
        let old_ids: Vec<CorpusId> = engine.corpora.keys().copied().collect();
        for id in old_ids {
            if id.0 as usize >= num_corpora {
                if let Some(corpus) = engine.corpora.remove(&id) {
                    for dot_id in corpus.dots.keys() {
                        engine.neurogenesis.unregister_dot(dot_id);
                    }
                }
            }
        }
        for index in 8..num_corpora {
            let id = CorpusId(index as u32);
            let name = format!("Custom-{index}");
            engine.corpora.insert(id, Corpus::new(id, &name, CorpusType::Association));
        }
        let valid: std::collections::HashSet<CorpusId> = engine.corpora.keys().copied().collect();
        engine.trans_channels.retain(|channel| {
            valid.contains(&channel.corpus_a) && valid.contains(&channel.corpus_b)
        });
        let ids: Vec<CorpusId> = engine.corpora.keys().copied().collect();
        for corpus_id in ids {
            let old_dots: Vec<DotId> = engine.corpora.get(&corpus_id)
                .map(|corpus| corpus.dots.keys().copied().collect())
                .unwrap_or_default();
            if let Some(corpus) = engine.corpora.get_mut(&corpus_id) {
                corpus.dots.clear();
                corpus.gates.clear();
                corpus.clumps.clear();
                corpus.bundles.clear();
            }
            for dot in old_dots {
                engine.neurogenesis.unregister_dot(&dot);
            }
            for _ in 0..dots_per_corpus {
                engine.spawn_dot(corpus_id);
            }
            engine.connect_corpus(corpus_id);
        }
        engine.neurogenesis.config_mut().max_dots_per_corpus = dots_per_corpus;
        engine.neurogenesis_enabled = enable_neurogenesis;
        engine.state = SystemState::Awake;
        Ok(engine)
    }

    fn connect_corpus(&mut self, corpus_id: CorpusId) {
        let dot_ids: Vec<DotId> = match self.corpora.get(&corpus_id) {
            Some(corpus) => corpus.dots.keys().copied().collect(),
            None => return,
        };
        let len = dot_ids.len();
        if len < 2 {
            return;
        }
        for i in 0..len {
            for j in 1..=(3 + (i % 4)) {
                let target_idx = (i + j * 7 + j * j) % len;
                if target_idx == i {
                    continue;
                }
                let source = dot_ids[i];
                let target = dot_ids[target_idx];
                let gate = Gate::soft();
                let gate_id = gate.id;
                if let Some(corpus) = self.corpora.get_mut(&corpus_id) {
                    corpus.gates.insert(gate_id, gate);
                    if let Some(dot) = corpus.dots.get_mut(&source) {
                        dot.add_branch(target, corpus_id, gate_id, 0.3 + (i as f64 * 0.01) % 0.7);
                    }
                    if let Some(dot) = corpus.dots.get_mut(&target) {
                        dot.add_ending(source, gate_id, 0.3 + (i as f64 * 0.01) % 0.7);
                    }
                }
            }
        }
    }

    /// Whether the engine owns the specified corpus.
    pub fn has_corpus(&self, id: CorpusId) -> bool {
        self.corpora.contains_key(&id)
    }

    /// Read motor-cortex potentials in stable dot-ID order.
    pub fn read_motor_output(&self) -> Vec<f32> {
        let mut values: Vec<(DotId, f32)> = self.corpora.get(&CorpusId(5))
            .map(|corpus| corpus.dots.values().map(|dot| (dot.id, dot.potential as f32)).collect())
            .unwrap_or_default();
        values.sort_by_key(|(id, _)| id.0);
        values.into_iter().map(|(_, value)| value).collect()
    }

    /// Apply a global reward signal through the existing reward system.
    pub fn apply_reward(&mut self, reward: f32) {
        self.reward(reward.clamp(-1.0, 1.0) as f64, "sdk");
    }

    /// Enter dream state.
    pub fn enter_dream(&mut self) {
        self.enter_sleep();
    }

    /// Return to awake state.
    pub fn wake(&mut self) {
        self.wake_up();
    }

    /// Snapshot average neurochemical levels across all dots.
    pub fn chemical_snapshot(&self) -> ChemicalSnapshot {
        let mut totals = [0.0f64; 6];
        let mut count = 0usize;
        for corpus in self.corpora.values() {
            for dot in corpus.dots.values() {
                totals[0] += dot.chemicals.dopamine;
                totals[1] += dot.chemicals.serotonin;
                totals[2] += dot.chemicals.cortisol;
                totals[3] += dot.chemicals.acetylcholine;
                totals[4] += dot.chemicals.gaba;
                totals[5] += dot.chemicals.glutamate;
                count += 1;
            }
        }
        if count == 0 {
            return ChemicalSnapshot::default();
        }
        let scale = 1.0 / count as f64;
        ChemicalSnapshot::from_values(
            (totals[0] * scale) as f32,
            (totals[1] * scale) as f32,
            (totals[2] * scale) as f32,
            (totals[3] * scale) as f32,
            (totals[4] * scale) as f32,
            (totals[5] * scale) as f32,
        )
    }

    /// Count dots that fired during the current engine tick.
    pub fn active_dot_count(&self) -> usize {
        self.corpora.values().flat_map(|corpus| corpus.dots.values())
            .filter(|dot| dot.recent_fire_ticks.last().copied() == Some(self.tick))
            .count()
    }

    /// Count pulse deliveries emitted by firing during the previous tick.
    pub fn propagated_pulse_count(&self) -> usize {
        self.last_propagated_pulses
    }

    /// Reset neurochemistry while preserving learned structure.
    pub fn reset_chemicals(&mut self) {
        for corpus in self.corpora.values_mut() {
            corpus.ambient_chemicals = ChemicalState::resting();
            for dot in corpus.dots.values_mut() {
                dot.chemicals = ChemicalState::resting();
            }
        }
        self.reward_system.global_reward = 0.0;
    }

    /// Inject a complete pulse into a chosen corpus, mapping its origin to a local dot.
    pub fn inject_pulse_object(&mut self, corpus_id: CorpusId, mut pulse: Pulse) -> Option<DotId> {
        let corpus = self.corpora.get(&corpus_id)?;
        let mut available: Vec<DotId> = corpus.dots.keys().copied().collect();
        if available.is_empty() {
            return None;
        }
        available.sort_unstable();
        let target = if corpus.dots.contains_key(&pulse.origin) {
            pulse.origin
        } else {
            available[(pulse.origin.0 % available.len() as u64) as usize]
        };
        pulse.origin = target;
        let gate = Gate::soft();
        let delivery_tick = pulse.created_tick.max(self.tick.saturating_add(1));
        self.pulse_queue.push_back(PulseDelivery {
            pulse,
            target_dot: target,
            target_corpus: corpus_id,
            target_gate: gate.id,
            delivery_tick,
        });
        Some(target)
    }

    /// Serialize the engine's structural and dynamic state for SDK persistence.
    pub fn serialize_state(&self) -> Result<Vec<u8>, String> {
        #[derive(serde::Serialize)]
        struct SnapshotRef<'a> {
            corpora: &'a HashMap<CorpusId, Corpus>,
            trans_channels: &'a Vec<TransBinaryChannel>,
            pulse_queue: &'a VecDeque<PulseDelivery>,
            tick: u64,
            state: &'a SystemState,
            dream_engine: &'a DreamEngine,
            hippocampus: &'a HippocampalEngine,
            reward_system: &'a RewardSystem,
            neurogenesis: &'a NeurogenesisController,
            multimodal_dot_map: &'a HashMap<DotId, DotId>,
            neurogenesis_enabled: bool,
            last_propagated_pulses: usize,
        }
        serde_json::to_vec(&SnapshotRef {
            corpora: &self.corpora,
            trans_channels: &self.trans_channels,
            pulse_queue: &self.pulse_queue,
            tick: self.tick,
            state: &self.state,
            dream_engine: &self.dream_engine,
            hippocampus: &self.hippocampus,
            reward_system: &self.reward_system,
            neurogenesis: &self.neurogenesis,
            multimodal_dot_map: &self.multimodal_dot_map,
            neurogenesis_enabled: self.neurogenesis_enabled,
            last_propagated_pulses: self.last_propagated_pulses,
        }).map_err(|error| error.to_string())
    }

    /// Restore engine state from a snapshot produced by serialize_state.
    pub fn deserialize_state(&mut self, bytes: &[u8]) -> Result<(), String> {
        #[derive(serde::Deserialize)]
        struct SnapshotOwned {
            corpora: HashMap<CorpusId, Corpus>,
            trans_channels: Vec<TransBinaryChannel>,
            pulse_queue: VecDeque<PulseDelivery>,
            tick: u64,
            state: SystemState,
            dream_engine: DreamEngine,
            hippocampus: HippocampalEngine,
            reward_system: RewardSystem,
            neurogenesis: NeurogenesisController,
            multimodal_dot_map: HashMap<DotId, DotId>,
            neurogenesis_enabled: bool,
            last_propagated_pulses: usize,
        }
        let snapshot: SnapshotOwned = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        self.corpora = snapshot.corpora;
        self.trans_channels = snapshot.trans_channels;
        self.pulse_queue = snapshot.pulse_queue;
        self.tick = snapshot.tick;
        self.state = snapshot.state;
        self.dream_engine = snapshot.dream_engine;
        self.hippocampus = snapshot.hippocampus;
        self.reward_system = snapshot.reward_system;
        self.neurogenesis = snapshot.neurogenesis;
        self.multimodal_dot_map = snapshot.multimodal_dot_map;
        self.neurogenesis_enabled = snapshot.neurogenesis_enabled;
        self.last_propagated_pulses = snapshot.last_propagated_pulses;
        Ok(())
    }

    pub fn status(&self) -> String {
        format!(
            "Tick: {} | State: {:?} | Dots: {} | Connections: {} | Clumps: {} | Pulses: {}",
            self.tick,
            self.state,
            self.total_dots(),
            self.total_connections(),
            self.total_clumps(),
            self.pending_pulses(),
        )
    }

    /// Inject a pulse directly into an existing dot (used by input encoders).
    pub fn inject_pulse(
        &mut self,
        target_dot: DotId,
        target_corpus: CorpusId,
        value: f64,
        delay_ticks: u64,
    ) {
        let pulse = Pulse::value_pulse(target_dot, value, 1.0, self.tick);
        let gate = Gate::soft();
        self.pulse_queue.push_back(PulseDelivery {
            pulse,
            target_dot,
            target_corpus,
            target_gate: gate.id,
            delivery_tick: self.tick + delay_ticks,
        });
    }

    /// Route a multi-modal pulse's logical sensory address to a physical dot.
    ///
    /// A dedicated physical dot is allocated for each logical address until
    /// the configured sensory-corpus dot cap is reached. At capacity, stable
    /// hashing maps additional addresses onto existing sensory dots.
    /// Returns the physical target dot, or None if the sensory corpus is absent.
    pub fn inject_multimodal_pulse(&mut self, pulse: Pulse) -> Option<DotId> {
        let sensory_corpus = CorpusId(0);
        if !self.corpora.contains_key(&sensory_corpus) {
            return None;
        }

        let logical_address = pulse.origin;
        let mapped_dot = self
            .multimodal_dot_map
            .get(&logical_address)
            .copied()
            .filter(|mapped| {
                self.corpora
                    .get(&sensory_corpus)
                    .map(|corpus| corpus.dots.contains_key(mapped))
                    .unwrap_or(false)
            });

        let target_dot = if let Some(mapped) = mapped_dot {
            mapped
        } else {
            self.multimodal_dot_map.remove(&logical_address);
            let current_count = self.corpora[&sensory_corpus].dot_count();
            let maximum = self.neurogenesis.config().max_dots_per_corpus;

            let mapped = if maximum == 0 || current_count < maximum {
                self.spawn_dot(sensory_corpus)
            } else {
                let mut available: Vec<DotId> = self.corpora[&sensory_corpus]
                    .dots
                    .keys()
                    .copied()
                    .collect();
                available.sort_unstable();
                if available.is_empty() {
                    return None;
                }

                let mut hash = logical_address.0;
                hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                hash ^= hash >> 31;
                available[(hash % available.len() as u64) as usize]
            };

            self.multimodal_dot_map.insert(logical_address, mapped);
            mapped
        };

        let gate = Gate::soft();
        let delivery_tick = pulse.created_tick.max(self.tick.saturating_add(1));
        self.pulse_queue.push_back(PulseDelivery {
            pulse,
            target_dot,
            target_corpus: sensory_corpus,
            target_gate: gate.id,
            delivery_tick,
        });

        Some(target_dot)
    }

    /// Route every pulse in a multi-modal bundle. Returns the number queued.
    pub fn inject_multimodal_input(
        &mut self,
        input: bionetic_core::multimodal::MultiModalInput,
    ) -> usize {
        let mut routed = 0;
        for pulse in input.into_flat() {
            if self.inject_multimodal_pulse(pulse).is_some() {
                routed += 1;
            }
        }
        routed
    }
}

impl Default for BioneticEngine {
    fn default() -> Self {
        Self::new()
    }
}
