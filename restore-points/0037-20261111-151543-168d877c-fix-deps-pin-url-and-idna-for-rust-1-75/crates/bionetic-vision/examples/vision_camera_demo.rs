//! Live camera → vision pipeline → SDK demo.
//! Run with: cargo run -p bionetic-vision --example vision_camera_demo --features camera

#[cfg(feature = "camera")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bionetic_sdk::BrainBuilder;
    use bionetic_vision::pipeline::{VisionPipeline, VisionPipelineConfig};
    use bionetic_vision::source::VisualSource;
    use bionetic_vision::sources::CameraSource;

    println!("Enumerating cameras...");
    for device in CameraSource::enumerate()? {
        println!("  {device}");
    }
    let mut camera = CameraSource::open_default(64, 48)?;
    println!("Opened: {} at {:?}", camera.name(), camera.dimensions());

    let mut pipeline = VisionPipeline::new(VisionPipelineConfig::default());
    let mut brain = BrainBuilder::new()
        .with_corpora(8)
        .with_dots_per_corpus(256)
        .with_max_input_pulses(16_384)
        .build();

    for tick in 1..=20 {
        let frame = camera.capture_frame()?;
        let pulses = pipeline.process_frame(&frame, tick);
        let output = brain.inject_multimodal_pulses(pulses)?;
        println!(
            "tick {tick}: captured {}x{}, motor outputs {}, active dots {}",
            frame.width,
            frame.height,
            output.motor.len(),
            output.active_dots
        );
        brain.idle(1);
    }
    camera.close()?;
    Ok(())
}

#[cfg(not(feature = "camera"))]
fn main() {
    eprintln!("Build this example with --features camera.");
}
