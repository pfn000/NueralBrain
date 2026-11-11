//! Live camera → vision pipeline → SDK demo.
//! Run with: cargo run -p bionetic-vision --example vision_camera_demo --features camera

#[cfg(feature = "camera")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bionetic_sdk::BrainBuilder;
    use bionetic_core::VISION;
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
        .with_corpora(bionetic_core::CORPUS_COUNT_TOTAL)
        .with_dots_per_corpus(256)
        .with_max_input_pulses(16_384)
        .build();

    for tick in 1..=20 {
        let frame = camera.capture_frame()?;
        let pulses = pipeline.process_frame(&frame, tick);
        let mut queued = 0usize;
        for pulse in pulses {
            if brain.engine_mut().inject_pulse_object(VISION, pulse).is_some() {
                queued += 1;
            }
        }
        brain.idle(1);
        let motor_outputs = brain.engine().read_motor_output().len();
        let active_dots = brain.engine().active_dot_count();
        println!(
            "tick {tick}: captured {}x{}, queued vision pulses {}, motor outputs {}, active dots {}",
            frame.width,
            frame.height,
            queued,
            motor_outputs,
            active_dots
        );
    }
    camera.close()?;
    Ok(())
}

#[cfg(not(feature = "camera"))]
fn main() {
    eprintln!("Build this example with --features camera.");
}
