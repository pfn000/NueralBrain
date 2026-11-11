use std::collections::HashMap;
use bionetic_core::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RewardSystem {
    pub global_reward: f64,
    pub action_outcomes: HashMap<String, f64>,
}

impl RewardSystem {
    pub fn new() -> Self {
        RewardSystem {
            global_reward: 0.0,
            action_outcomes: HashMap::new(),
        }
    }

    pub fn reward(
        &mut self,
        magnitude: f64,
        behavior: &str,
        corpora: &mut HashMap<CorpusId, Corpus>,
        _tick: u64,
    ) {
        let current = self.action_outcomes.entry(behavior.to_string()).or_insert(0.0);
        *current = (*current + magnitude) / 2.0;

        // Dopamine cascade: boost recently active dots
        for corpus in corpora.values_mut() {
            for dot in corpus.dots.values_mut() {
                if dot.fire_count > 0 {
                    dot.chemicals.dopamine += magnitude * 0.3;
                    dot.value += magnitude * 0.05;

                    for branch in &mut dot.branches {
                        if branch.transmission_count > 0 {
                            branch.strength = (branch.strength + magnitude * 0.05).min(5.0);
                            branch.success_rate = (branch.success_rate + 0.02).min(1.0);
                        }
                    }
                }
            }
        }

        self.global_reward = (self.global_reward + magnitude) / 2.0;
    }

    pub fn discipline(
        &mut self,
        magnitude: f64,
        behavior: &str,
        corpora: &mut HashMap<CorpusId, Corpus>,
        _tick: u64,
    ) {
        let current = self.action_outcomes.entry(behavior.to_string()).or_insert(0.0);
        *current = (*current - magnitude) / 2.0;

        // Cortisol cascade
        for corpus in corpora.values_mut() {
            for dot in corpus.dots.values_mut() {
                if dot.fire_count > 0 {
                    dot.chemicals.cortisol += magnitude * 0.3;
                    dot.chemicals.dopamine = (dot.chemicals.dopamine - magnitude * 0.1).max(0.0);

                    for branch in &mut dot.branches {
                        if branch.transmission_count > 0 {
                            branch.strength = (branch.strength - magnitude * 0.08).max(0.01);
                            branch.success_rate = (branch.success_rate - 0.05).max(0.0);
                        }
                    }
                }
            }
        }
    }
}

impl Default for RewardSystem {
    fn default() -> Self {
        Self::new()
    }
}
