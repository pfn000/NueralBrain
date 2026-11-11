//! Integration tests for the neurogenesis system.

use bionetic_core::dot::DotId;
use bionetic_core::ids::CorpusId;
use bionetic_core::neurogenesis::{
    NeuroEvent, NeurogenesisConfig, NeurogenesisController,
};

#[test]
fn test_full_growth_cycle() {
    let config = NeurogenesisConfig {
        growth_threshold: 0.3,
        growth_patience: 2,
        spawn_batch: 5,
        max_dots_per_corpus: 200,
        eval_interval: 10,
        ..Default::default()
    };

    let mut neuro = NeurogenesisController::new(config, &[]);
    neuro.set_dot_count(CorpusId(0), 10);
    let mut total_growth = 0;

    for tick in 1..=100 {
        for _ in 0..5 {
            neuro.record_pulse(CorpusId(0));
        }

        for event in neuro.tick(tick) {
            if let NeuroEvent::Growth { count, .. } = event {
                total_growth += count;
            }
        }
    }

    assert!(
        total_growth > 0,
        "Expected at least one growth event in 100 ticks"
    );
    println!("Total dots grown: {total_growth}");
}

#[test]
fn test_newborn_integration() {
    let config = NeurogenesisConfig {
        eval_interval: 5,
        growth_threshold: 0.1,
        growth_patience: 1,
        spawn_batch: 2,
        ..Default::default()
    };

    let mut neuro = NeurogenesisController::new(config, &[]);
    neuro.set_dot_count(CorpusId(1), 5);

    for tick in 1..=10 {
        for _ in 0..3 {
            neuro.record_pulse(CorpusId(1));
        }

        for event in neuro.tick(tick) {
            if let NeuroEvent::Growth { corpus, count, tick } = event {
                let newborns = neuro.create_newborn_dots(corpus, count, tick);
                assert_eq!(newborns.len(), count);

                for dot in &newborns {
                    assert_eq!(dot.corpus, CorpusId(1));
                    assert_eq!(neuro.dot_lifetime_pulses(&dot.id), 0);
                }
            }
        }
    }
}

#[test]
fn test_multi_corpus_independence() {
    let config = NeurogenesisConfig {
        growth_threshold: 0.5,
        growth_patience: 1,
        spawn_batch: 2,
        eval_interval: 5,
        ..Default::default()
    };

    let mut neuro = NeurogenesisController::new(config, &[]);
    neuro.set_dot_count(CorpusId(0), 2);
    neuro.set_dot_count(CorpusId(1), 2);

    for tick in 1..=10 {
        for _ in 0..5 {
            neuro.record_pulse(CorpusId(0));
        }
        neuro.record_pulse(CorpusId(1));

        for event in neuro.tick(tick) {
            if let NeuroEvent::Growth { corpus, .. } = event {
                assert_eq!(corpus, CorpusId(0), "Only corpus 0 should grow");
            }
        }
    }
}

#[test]
fn test_serialization_roundtrip() {
    let config = NeurogenesisConfig::default();
    let seed = vec![(DotId(1), 0), (DotId(2), 0), (DotId(3), 0)];
    let neuro = NeurogenesisController::new(config, &seed);

    let json = serde_json::to_string(&neuro).expect("serialize");
    let restored: NeurogenesisController =
        serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.tracked_dot_count(), 3);
}
