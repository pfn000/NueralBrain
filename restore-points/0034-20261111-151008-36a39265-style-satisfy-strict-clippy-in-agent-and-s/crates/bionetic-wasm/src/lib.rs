use wasm_bindgen::prelude::*;
use std::sync::Mutex;
use bionetic_engine::BioneticEngine;
use bionetic_io::{InputEncoder, OutputDecoder};
use bionetic_io::serialization::serialize_status;

static ENGINE: Mutex<Option<BioneticEngine>> = Mutex::new(None);
static ENCODER: Mutex<Option<InputEncoder>> = Mutex::new(None);

#[wasm_bindgen(start)]
pub fn wasm_main() {
    // Optional: set up panic hook for better error messages
}

#[wasm_bindgen]
pub fn init_brain() -> String {
    let engine = BioneticEngine::new();
    let status = engine.status();

    *ENGINE.lock().unwrap() = Some(engine);
    *ENCODER.lock().unwrap() = Some(InputEncoder::new());

    status
}

#[wasm_bindgen]
pub fn process_input(input: &str) -> String {
    let mut engine_lock = ENGINE.lock().unwrap();
    let mut encoder_lock = ENCODER.lock().unwrap();

    if let (Some(engine), Some(encoder)) = (engine_lock.as_mut(), encoder_lock.as_mut()) {
        encoder.encode_input(input, engine);

        // Process for up to 500 ticks
        for _ in 0..500 {
            engine.tick();
        }

        let decoder = OutputDecoder::new();
        decoder.decode_output(engine)
    } else {
        "Brain not initialized".to_string()
    }
}

#[wasm_bindgen]
pub fn tick_brain() -> String {
    let mut engine_lock = ENGINE.lock().unwrap();
    if let Some(engine) = engine_lock.as_mut() {
        engine.tick();
        serialize_status(engine)
    } else {
        "{}".to_string()
    }
}

#[wasm_bindgen]
pub fn trigger_reward(magnitude: f64, behavior: &str) {
    let mut engine_lock = ENGINE.lock().unwrap();
    if let Some(engine) = engine_lock.as_mut() {
        engine.reward(magnitude, behavior);
    }
}

#[wasm_bindgen]
pub fn trigger_discipline(magnitude: f64, behavior: &str) {
    let mut engine_lock = ENGINE.lock().unwrap();
    if let Some(engine) = engine_lock.as_mut() {
        engine.discipline(magnitude, behavior);
    }
}

#[wasm_bindgen]
pub fn enter_sleep() {
    let mut engine_lock = ENGINE.lock().unwrap();
    if let Some(engine) = engine_lock.as_mut() {
        engine.enter_sleep();
    }
}

#[wasm_bindgen]
pub fn wake_up() {
    let mut engine_lock = ENGINE.lock().unwrap();
    if let Some(engine) = engine_lock.as_mut() {
        engine.wake_up();
    }
}

#[wasm_bindgen]
pub fn get_brain_state() -> String {
    let engine_lock = ENGINE.lock().unwrap();
    if let Some(engine) = engine_lock.as_ref() {
        serialize_status(engine)
    } else {
        "{}".to_string()
    }
}

#[wasm_bindgen]
pub fn get_status_line() -> String {
    let engine_lock = ENGINE.lock().unwrap();
    if let Some(engine) = engine_lock.as_ref() {
        engine.status()
    } else {
        "Not initialized".to_string()
    }
}
