//! Demo: two local brains exchange pulses through the brain-to-brain protocol.
//! In a real deployment, replace the direct byte-passing with a WebSocket,
//! WebRTC DataChannel, or TCP socket.

use bionetic_core::ids::DotId;
use bionetic_core::ids::CorpusId;
use bionetic_core::{SENSORY_IN, SENSORY_OUT, VISION, PREFRONTAL};
use bionetic_core::pulse::Pulse;
use bionetic_engine::brain_protocol::{BrainNetwork, PROTOCOL_VERSION};

fn main() {
    println!("=== Two-Brain Communication Demo ===");
    println!("Protocol version: {PROTOCOL_VERSION}\n");

    // Create two brain networks
    let mut brain_a = BrainNetwork::new(vec![SENSORY_IN, SENSORY_OUT], 100);
    let mut brain_b = BrainNetwork::new(vec![VISION, PREFRONTAL], 100);

    brain_a.advertise_feature("vision");
    brain_b.advertise_feature("reasoning");

    let a_id = brain_a.local_id();
    let b_id = brain_b.local_id();
    println!("Brain A: {a_id}");
    println!("Brain B: {b_id}\n");

    // --- Handshake ---
    println!("[handshake] A → B: Hello");
    brain_a.initiate_handshake(b_id);
    relay(&mut brain_a, &mut brain_b);

    println!("[handshake] B → A: Ack");
    relay(&mut brain_b, &mut brain_a);

    println!("A peers: {}, B peers: {}\n", brain_a.peer_count(), brain_b.peer_count());

    // --- Pulse exchange ---
    for tick in 1..=5 {
        brain_a.set_tick(tick);
        brain_b.set_tick(tick);

        let pulses: Vec<Pulse> = (0..3)
            .map(|i| Pulse::value_pulse(
                DotId(i),
                0.5 + i as f64 * 0.1,
                0.5 + i as f64 * 0.1,
                tick,
            ))
            .collect();

        println!("[tick {tick}] A → B: {} pulses to VISION", pulses.len());
        brain_a.send_pulses(b_id, VISION, pulses);
        relay(&mut brain_a, &mut brain_b);

        let delivered = brain_b.drain_inbox_pulses();
        println!("[tick {tick}] B inbox: {} pulses received\n", delivered.len());
    }

    // --- Graceful disconnect ---
    println!("[disconnect] A says goodbye");
    brain_a.disconnect(b_id, Some("demo complete".to_string()));
    relay(&mut brain_a, &mut brain_b);

    println!("Final peer counts — A: {}, B: {}", brain_a.peer_count(), brain_b.peer_count());
}

/// Transport stub: ship outbox bytes from `from` into `to`.
fn relay(from: &mut BrainNetwork, to: &mut BrainNetwork) {
    for (_, bytes) in from.drain_outbox() {
        to.receive_bytes(&bytes);
    }
}