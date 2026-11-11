use std::collections::HashMap;
use bionetic_core::*;
use bionetic_engine::BioneticEngine;

pub struct InputEncoder {
    pub encoding_map: HashMap<char, Vec<(DotId, f64)>>,
}

impl InputEncoder {
    pub fn new() -> Self {
        InputEncoder {
            encoding_map: HashMap::new(),
        }
    }

    pub fn encode_input(&mut self, input: &str, engine: &mut BioneticEngine) {
        let sensory_id = CorpusId(0);

        for (i, ch) in input.chars().enumerate() {
            let activations = self.get_or_create_encoding(ch, engine);

            for (dot_id, strength) in activations {
                engine.inject_pulse(
                    dot_id,
                    sensory_id,
                    strength * 20.0,
                    (i as u64) + 1,
                );
            }
        }
    }

    fn get_or_create_encoding(
        &mut self,
        ch: char,
        engine: &mut BioneticEngine,
    ) -> Vec<(DotId, f64)> {
        if let Some(existing) = self.encoding_map.get(&ch) {
            return existing.clone();
        }

        let sensory_id = CorpusId(0);
        let pattern_size = 4 + (ch as usize % 5);
        let mut encoding = Vec::new();

        for i in 0..pattern_size {
            let dot_id = engine.spawn_dot(sensory_id);
            let strength = 0.5 + ((i as f64 * 0.7).sin().abs() * 0.5);
            encoding.push((dot_id, strength));
        }

        self.encoding_map.insert(ch, encoding.clone());
        encoding
    }
}

impl Default for InputEncoder {
    fn default() -> Self {
        Self::new()
    }
}
