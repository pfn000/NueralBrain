//! Engine integration tests for logical multi-modal sensory routing.

use bionetic_core::ids::CorpusId;
use bionetic_core::multimodal::{
    audio_bands_to_pulses, image_grayscale_to_pulses, Modality, MultiModalInput,
};
use bionetic_engine::BioneticEngine;

#[test]
fn test_multimodal_pulse_routes_to_physical_sensory_dot() {
    let mut brain = BioneticEngine::new();
    let initial_dots = brain.total_dots();
    let pulse = image_grayscale_to_pulses(&[255], 1, 1, 1, 0)
        .into_iter()
        .next()
        .expect("one bright pixel produces a pulse");
    let logical_address = pulse.origin;

    let target = brain
        .inject_multimodal_pulse(pulse)
        .expect("sensory corpus exists");

    assert_ne!(target, logical_address);
    assert!(brain.corpora[&CorpusId(0)].dots.contains_key(&target));
    assert_eq!(brain.total_dots(), initial_dots + 1);
    assert_eq!(brain.pending_pulses(), 1);

    brain.tick();

    assert_eq!(brain.pending_pulses(), 0);
    assert!(brain.corpora[&CorpusId(0)].dots[&target].potential > -70.0);
}

#[test]
fn test_logical_address_reuses_same_sensory_dot() {
    let mut brain = BioneticEngine::new();
    let initial_dots = brain.total_dots();

    let first = image_grayscale_to_pulses(&[255], 1, 1, 1, 0).remove(0);
    let second = image_grayscale_to_pulses(&[128], 1, 1, 2, 0).remove(0);
    let first_target = brain.inject_multimodal_pulse(first).unwrap();
    let second_target = brain.inject_multimodal_pulse(second).unwrap();

    assert_eq!(first_target, second_target);
    assert_eq!(brain.total_dots(), initial_dots + 1);
}

#[test]
fn test_sensory_capacity_uses_stable_fallback_mapping() {
    let mut brain = BioneticEngine::new();
    brain.neurogenesis_mut().config_mut().max_dots_per_corpus = 64;
    let initial_dots = brain.total_dots();

    let vision = image_grayscale_to_pulses(&[255], 1, 1, 1, 0).remove(0);
    let audio = audio_bands_to_pulses(&[0.9], 1, 0.0).remove(0);
    let vision_target = brain.inject_multimodal_pulse(vision).unwrap();
    let audio_target = brain.inject_multimodal_pulse(audio).unwrap();

    assert!(brain.corpora[&CorpusId(0)].dots.contains_key(&vision_target));
    assert!(brain.corpora[&CorpusId(0)].dots.contains_key(&audio_target));
    assert_eq!(brain.total_dots(), initial_dots);
}

#[test]
fn test_multimodal_bundle_routes_all_modalities() {
    let mut brain = BioneticEngine::new();
    let mut input = MultiModalInput::new(1);
    input
        .add(
            Modality::Vision,
            image_grayscale_to_pulses(&[255, 128], 2, 1, 1, 0),
        )
        .add(Modality::Audio, audio_bands_to_pulses(&[0.5], 1, 0.0));

    let expected = input.total_pulses();
    let routed = brain.inject_multimodal_input(input);

    assert_eq!(routed, expected);
    assert_eq!(brain.pending_pulses(), expected);
}
