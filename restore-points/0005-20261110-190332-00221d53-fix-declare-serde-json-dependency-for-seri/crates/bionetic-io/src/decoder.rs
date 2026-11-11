use bionetic_core::*;
use bionetic_engine::BioneticEngine;

pub struct OutputDecoder {
    pub match_threshold: f64,
}

impl OutputDecoder {
    pub fn new() -> Self {
        OutputDecoder {
            match_threshold: 0.6,
        }
    }

    pub fn decode_output(&self, engine: &BioneticEngine) -> String {
        let motor_id = CorpusId(5);

        if let Some(motor) = engine.corpora.get(&motor_id) {
            // Check for resolved clumps
            for clump in motor.clumps.values() {
                if clump.resolved {
                    if let Some(output) = &clump.output {
                        return output.resolution.clone();
                    }
                }
            }

            // Fallback: report active dot count
            let active = motor.active_dots(engine.tick, 10);
            if !active.is_empty() {
                return format!("[{} motor dots active]", active.len());
            }
        }

        String::from("[no output]")
    }
}

impl Default for OutputDecoder {
    fn default() -> Self {
        Self::new()
    }
}
