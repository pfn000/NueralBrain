//! Demo: feed vision, audio, and text into one brain concurrently.

use bionetic_core::multimodal::{
    audio_bands_to_pulses, image_grayscale_to_pulses, text_to_pulses, Modality,
    MultiModalInput,
};
use bionetic_engine::BioneticEngine;

fn main() {
    println!("=== NeuralBrain Multi-modal Demo ===\n");

    let mut brain = BioneticEngine::new();

    // Fake sensory input: a bright image, a loud tone, and the text "hi there".
    let fake_image: Vec<u8> = (0..64).map(|index| (index * 4) as u8).collect();
    let fake_audio = vec![0.1_f32, 0.3, 0.8, 0.9, 0.7, 0.2, 0.1];
    let text = "hi there";

    for tick in 1..=50 {
        let mut input = MultiModalInput::new(tick);
        input
            .add(
                Modality::Vision,
                image_grayscale_to_pulses(&fake_image, 8, 8, tick, 20),
            )
            .add(
                Modality::Audio,
                audio_bands_to_pulses(&fake_audio, tick, 0.05),
            )
            .add(Modality::Text, text_to_pulses(text, tick));

        let modality_count = input.modality_count();
        let pulse_count = input.total_pulses();
        let routed = brain.inject_multimodal_input(input);

        println!(
            "Tick {tick:>3}: modalities={modality_count} pulses={pulse_count} routed={routed}"
        );

        brain.tick();
    }

    println!("\n=== Done. Brain processed multi-modal input for 50 ticks. ===");
}
