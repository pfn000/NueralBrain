//! Basic example: create a brain, send input, process, reward, sleep.

use bionetic_engine::BioneticEngine;
use bionetic_io::{InputEncoder, OutputDecoder};

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          BIONETIC BRAIN — Initializing...                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let mut brain = BioneticEngine::new();
    println!("[BOOT] {}", brain.status());

    let mut encoder = InputEncoder::new();
    let decoder = OutputDecoder::new();

    // Send input
    println!("\n[INPUT] Encoding \"hello\"...");
    encoder.encode_input("hello", &mut brain);

    // Process
    println!("[PROC] Running 500 ticks...");
    for i in 0..500 {
        brain.tick();
        if i % 100 == 0 {
            println!("  tick {}: {}", i, brain.status());
        }
    }

    // Read output
    let output = decoder.decode_output(&brain);
    println!("\n[OUTPUT] {}", output);

    // Reward
    println!("\n[REWARD] Applying reward (1.0)...");
    brain.reward(1.0, "greeting_response");

    // Sleep
    println!("\n[SLEEP] Entering dream state...");
    brain.enter_sleep();
    for i in 0..200 {
        brain.tick();
        if i % 50 == 0 {
            println!("  dream tick {}: {}", i, brain.status());
        }
    }

    // Wake
    brain.wake_up();
    println!("\n[WAKE] {}", brain.status());

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          Session complete.                               ║");
    println!("╚══════════════════════════════════════════════════════════╝");
}
