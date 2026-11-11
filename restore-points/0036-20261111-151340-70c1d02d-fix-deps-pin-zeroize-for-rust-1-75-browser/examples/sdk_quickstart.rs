//! The simplest possible NeuralBrain app — 20 lines total.

use bionetic_sdk::{BrainBuilder, InputBatch, CorpusId};

fn main() {
    let mut brain = BrainBuilder::new()
        .with_corpora(8)
        .with_dots_per_corpus(128)
        .with_dreaming(true)
        .build();

    println!("Brain built with {} dots\n", brain.dot_count());

    // Think
    for i in 1..=5 {
        let input = vec![0.1 * i as f32, 0.5, 0.9];
        let out = brain.think_detailed(&input);
        println!(
            "tick {} → motor outputs: {} | active dots: {}",
            out.tick,
            out.motor.len(),
            out.active_dots
        );
    }

    // Learn with reward
    println!("\n--- Reinforcing pattern with reward ---");
    for _ in 0..10 {
        brain.learn(&[0.5, 0.5, 0.5], 0.9);
    }
    println!("Dopamine after reward: {:.3}", brain.chemicals().dopamine);

    // Dream
    println!("\n--- Dreaming for 100 ticks ---");
    brain.dream(100);
    println!("Tick after dream: {}", brain.tick());

    // Save / load
    let saved = brain.save();
    println!("\nSaved state: {} bytes", saved.len());
}