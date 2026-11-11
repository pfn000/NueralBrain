use std::collections::{HashMap, VecDeque};
use bionetic_core::*;
use crate::hippocampus::HippocampalEngine;
use crate::reward::RewardSystem;
use crate::dream::DreamEngine;

/// A pulse scheduled for delivery
#[derive(Debug, Clone)]
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

        if let Some(corpus) = self.corpora.get_mut(&corpus_id) {
            corpus.dots.insert(id, dot);
        }

        id
    }

    pub fn tick(&mut self) {
        self.tick += 1;

        match self.state {
            SystemState::Awake => self.tick_awake(),
            SystemState::Drowsy => self.tick_drowsy(),
            SystemState::Dreaming => self.tick_dreaming(),
            SystemState::DeepSleep => self.tick_deep_sleep(),
            SystemState::Booting => {}
        }
    }

    // Keep modulo checks compatible with the documented Rust 1.75+ floor.
    #[allow(clippy::manual_is_multiple_of)]
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

        while let Some(delivery) = self.pulse_queue.pop_front() {
            if delivery.delivery_tick <= self.tick {
                if let Some(corpus) = self.corpora.get_mut(&delivery.target_corpus) {
                    if let Some(dot) = corpus.dots.get_mut(&delivery.target_dot) {
                        // Apply pulse to dot
                        dot.potential += delivery.pulse.value;
                        dot.chemicals.apply_delta(&delivery.pulse.chemical_payload);
                    }
                }
            } else {
                remaining.push_back(delivery);
            }
        }

        self.pulse_queue = remaining;
    }

    fn process_corpus(&mut self, corpus_id: CorpusId) {
        let mut new_deliveries: Vec<PulseDelivery> = Vec::new();

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

        for delivery in new_deliveries {
            self.pulse_queue.push_back(delivery);
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

    /// Inject a pulse directly into a dot (used by input encoder)
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
}

impl Default for BioneticEngine {
    fn default() -> Self {
        Self::new()
    }
}
