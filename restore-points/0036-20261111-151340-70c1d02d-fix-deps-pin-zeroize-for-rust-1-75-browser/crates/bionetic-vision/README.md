# bionetic-vision

Vision processing pipeline for NeuralBrain. Converts frames from a webcam, browser adapter, or image file into pulse trains for the sensory cortex.

## Pipeline

\`\`\`text
Frame ──▶ Edge Detection ──┐
       ──▶ Color Channels ──┼──▶ Context Binding ──▶ Pulses
       ──▶ Motion Delta ────┘
\`\`\`

## Quick start

\`\`\`rust
use bionetic_sdk::BrainBuilder;
use bionetic_vision::pipeline::{VisionPipeline, VisionPipelineConfig};
use bionetic_vision::source::VisualSource;
use bionetic_vision::sources::CameraSource;

let mut camera = CameraSource::open_default(64, 48)?;
let mut pipeline = VisionPipeline::new(VisionPipelineConfig::default());
let mut brain = BrainBuilder::new()
    .with_max_input_pulses(16_384)
    .build();

let frame = camera.capture_frame()?;
let pulses = pipeline.process_frame(&frame, 1);
brain.inject_multimodal_pulses(pulses)?;
\`\`\`

## Features

| Feature | Source |
|---|---|
| \`camera\` | Native webcam capture through nokhwa (V4L2, MSMF, AVFoundation) |
| \`iris\` | Trait-compatible local stub; requires the real Iris repository/API to become operational |
| \`file-source\` | Image files and recorded frame sequences |
| \`full\` | All of the above |

The Iris adapter intentionally does not fetch an unknown crate or claim to capture a browser. Its local launcher returns a clear stub error until the actual Iris project URL and API are known.

## Build and test

\`\`\`bash
cargo build -p bionetic-vision
cargo test -p bionetic-vision
cargo check -p bionetic-vision --features file-source
cargo check -p bionetic-vision --features camera
cargo check -p bionetic-vision --features full
cargo run -p bionetic-vision --example vision_camera_demo --features camera
\`\`\`

A Linux camera build may require the V4L2 and udev development packages. CI compiles the camera adapter but does not require a physical camera.
