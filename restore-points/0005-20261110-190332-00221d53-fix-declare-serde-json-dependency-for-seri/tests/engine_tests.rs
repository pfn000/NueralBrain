use bionetic_engine::BioneticEngine;
use bionetic_io::InputEncoder;

#[test]
fn test_brain_initialization() {
    let brain = BioneticEngine::new();
    assert_eq!(brain.total_dots(), 512); // 64 per corpus × 8 corpora
    assert_eq!(brain.total_corpora(), 8);
    assert!(brain.total_connections() > 0);
}

#[test]
fn test_tick_processing() {
    let mut brain = BioneticEngine::new();

    for _ in 0..100 {
        brain.tick();
    }

    assert_eq!(brain.tick, 100);
}

#[test]
fn test_input_encoding() {
    let mut brain = BioneticEngine::new();
    let mut encoder = InputEncoder::new();

    let initial_dots = brain.total_dots();
    encoder.encode_input("test", &mut brain);

    // Should have created new dots for encoding
    assert!(brain.total_dots() > initial_dots);
    // Should have queued pulses
    assert!(brain.pending_pulses() > 0);
}

#[test]
fn test_reward_system() {
    let mut brain = BioneticEngine::new();

    // Process some ticks to get activity
    let mut encoder = InputEncoder::new();
    encoder.encode_input("hi", &mut brain);
    for _ in 0..50 {
        brain.tick();
    }

    // Reward should not panic
    brain.reward(1.0, "test_reward");
}

#[test]
fn test_sleep_cycle() {
    let mut brain = BioneticEngine::new();

    brain.enter_sleep();
    for _ in 0..100 {
        brain.tick();
    }

    brain.wake_up();
    assert_eq!(brain.state, bionetic_engine::engine::SystemState::Awake);
}

#[test]
fn test_discipline() {
    let mut brain = BioneticEngine::new();

    let mut encoder = InputEncoder::new();
    encoder.encode_input("bad", &mut brain);
    for _ in 0..50 {
        brain.tick();
    }

    brain.discipline(1.0, "test_discipline");
}
