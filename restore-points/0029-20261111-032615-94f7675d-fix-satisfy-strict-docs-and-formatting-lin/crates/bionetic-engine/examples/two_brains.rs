//! Demo of two local brains exchanging pulse bundles.
use bionetic_core::ids::{CorpusId, DotId};
use bionetic_core::pulse::Pulse;
use bionetic_engine::brain_protocol::{BrainNetwork, PROTOCOL_VERSION};

fn main() {
    println!("Protocol version: {PROTOCOL_VERSION}");
    let mut a = BrainNetwork::new(vec![CorpusId(0), CorpusId(1)], 100);
    let mut b = BrainNetwork::new(vec![CorpusId(2), CorpusId(3)], 100);
    let b_id = b.local_id();
    a.initiate_handshake(b_id);
    relay(&mut a, &mut b);
    relay(&mut b, &mut a);
    for tick in 1..=5 {
        a.set_tick(tick);
        let pulses = (0..3).map(|i| Pulse::value_pulse(DotId(i), 0.5, 0.5, tick)).collect();
        a.send_pulses(b_id, CorpusId(2), pulses);
        relay(&mut a, &mut b);
        println!("tick {tick}: received {}", b.drain_inbox_pulses().len());
    }
}
fn relay(from: &mut BrainNetwork, to: &mut BrainNetwork) {
    for (_, bytes) in from.drain_outbox() { let _ = to.receive_bytes(&bytes); }
}
