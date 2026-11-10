use std::collections::HashMap;
use bionetic_core::*;

#[derive(Debug, Clone, PartialEq)]
pub enum DreamPhase {
    Replay,
    Reflect,
    Consolidate,
    Prune,
    Synthesize,
    SelfJudge,
}

pub struct DreamEngine {
    pub phase: DreamPhase,
    pub cycle: u64,
}

impl DreamEngine {
    pub fn new() -> Self {
        DreamEngine {
            phase: DreamPhase::Replay,
            cycle: 0,
        }
    }

    pub fn dream_tick(&mut self, corpora: &mut HashMap<CorpusId, Corpus>, _tick: u64) {
        self.cycle += 1;

        match self.phase {
            DreamPhase::Replay => {
                // Strengthen connections of high-value dots
                for corpus in corpora.values_mut() {
                    for dot in corpus.dots.values_mut() {
                        if dot.value > 0.5 {
                            for branch in &mut dot.branches {
                                branch.strength = (branch.strength + 0.002).min(5.0);
                            }
                        }
                    }
                }
                self.phase = DreamPhase::Reflect;
            }
            DreamPhase::Reflect => {
                self.phase = DreamPhase::Consolidate;
            }
            DreamPhase::Consolidate => {
                self.phase = DreamPhase::Prune;
            }
            DreamPhase::Prune => {
                // Remove very weak, unused connections
                for corpus in corpora.values_mut() {
                    for dot in corpus.dots.values_mut() {
                        dot.branches.retain(|b| b.strength > 0.02 || b.transmission_count > 3);
                    }
                }
                self.phase = DreamPhase::Synthesize;
            }
            DreamPhase::Synthesize => {
                self.phase = DreamPhase::SelfJudge;
            }
            DreamPhase::SelfJudge => {
                // Strengthen high-success-rate pathways
                for corpus in corpora.values_mut() {
                    for dot in corpus.dots.values_mut() {
                        for branch in &mut dot.branches {
                            if branch.success_rate > 0.7 {
                                branch.strength = (branch.strength + 0.005).min(5.0);
                            } else if branch.success_rate < 0.2 {
                                branch.strength = (branch.strength - 0.005).max(0.01);
                            }
                        }
                    }
                }
                self.phase = DreamPhase::Replay;
            }
        }
    }
}
