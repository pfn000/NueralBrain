//! Extended example: train through multiple input/reward cycles with sleep.

use bionetic_engine::BioneticEngine;
use bionetic_io::InputEncoder;

fn main() {
    let mut brain = BioneticEngine::new();
    let mut encoder = InputEncoder::new();

    println!("Training session: 10 cycles with sleep between each\n");

    for cycle in 0..10 {
        println!("=== Cycle {} ===", cycle + 1);

        // Input
        encoder.encode_input("learn", &mut brain);

        // Process
        for _ in 0..200 {
            brain.tick();
        }

        // Reward
        brain.reward(0.5 + (cycle as f64 * 0.1), "learning");
        println!("  Rewarded. {}", brain.status());

        // Brief sleep
        brain.enter_sleep();
        for _ in 0..100 {
            brain.tick();
        }
        brain.wake_up();

        println!("  After sleep: {}", brain.status());
        println!();
    }

    println!("Final state: {}", brain.status());
}
