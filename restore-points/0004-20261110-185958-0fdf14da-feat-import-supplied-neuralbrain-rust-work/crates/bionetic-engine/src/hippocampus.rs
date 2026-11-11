use std::collections::{HashMap, VecDeque};
use bionetic_core::*;

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: u64,
    pub created_tick: u64,
    pub value: f64,
    pub access_count: u64,
    pub pathway: Vec<DotId>,
    pub tags: Vec<String>,
}

pub struct HippocampalEngine {
    pub short_term: VecDeque<MemoryEntry>,
    pub stm_capacity: usize,
    pub ltm_threshold: f64,
    next_id: u64,
}

impl HippocampalEngine {
    pub fn new() -> Self {
        HippocampalEngine {
            short_term: VecDeque::new(),
            stm_capacity: 256,
            ltm_threshold: 0.6,
            next_id: 1,
        }
    }

    pub fn store_short_term(&mut self, value: f64, pathway: Vec<DotId>, tags: Vec<String>, tick: u64) {
        let entry = MemoryEntry {
            id: self.next_id,
            created_tick: tick,
            value,
            access_count: 0,
            pathway,
            tags,
        };
        self.next_id += 1;
        self.short_term.push_back(entry);

        while self.short_term.len() > self.stm_capacity {
            self.short_term.pop_front();
        }
    }

    pub fn consolidate(&mut self, corpora: &mut HashMap<CorpusId, Corpus>) {
        let mut to_remove = Vec::new();

        for (idx, entry) in self.short_term.iter().enumerate() {
            if entry.value >= self.ltm_threshold || entry.access_count > 5 {
                // Strengthen pathway in the brain permanently
                for window in entry.pathway.windows(2) {
                    if let [source, target] = window {
                        for corpus in corpora.values_mut() {
                            if let Some(dot) = corpus.dots.get_mut(source) {
                                for branch in &mut dot.branches {
                                    if branch.target_dot == *target {
                                        branch.strength = (branch.strength + 0.02).min(5.0);
                                        branch.success_rate = (branch.success_rate + 0.01).min(1.0);
                                    }
                                }
                                dot.value += 0.01;
                            }
                        }
                    }
                }
                to_remove.push(idx);
            }
        }

        for idx in to_remove.into_iter().rev() {
            self.short_term.remove(idx);
        }
    }
}
