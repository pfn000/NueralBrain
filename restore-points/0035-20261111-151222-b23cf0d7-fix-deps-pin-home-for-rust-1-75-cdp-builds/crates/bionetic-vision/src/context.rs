//! Context binding — group co-occurring visual pulses into a synchronous clump.

use bionetic_core::clump::Clump;
use bionetic_core::ids::{ClumpId, DotId};
use bionetic_core::pulse::Pulse;

/// A bundle of visual-stage pulses bound into a synchronous clump.
#[derive(Debug)]
pub struct ContextBundle {
    /// Pulses created by edge detection.
    pub edge_pulses: Vec<Pulse>,
    /// Pulses created by color separation.
    pub color_pulses: Vec<Pulse>,
    /// Pulses created by temporal motion detection.
    pub motion_pulses: Vec<Pulse>,
    /// Clump binding the unique source dots from all stages.
    pub binding_clump: Clump,
}

impl ContextBundle {
    /// Create a context bundle from the three pipeline stages.
    pub fn bind(edge_pulses: Vec<Pulse>, color_pulses: Vec<Pulse>, motion_pulses: Vec<Pulse>, tick: u64) -> Self {
        let mut dot_ids: Vec<DotId> = edge_pulses.iter()
            .chain(color_pulses.iter())
            .chain(motion_pulses.iter())
            .map(|pulse| pulse.origin)
            .collect();
        dot_ids.sort_by_key(|dot| dot.0);
        dot_ids.dedup();

        let mut binding_clump = Clump::new(
            ClumpId::next(),
            dot_ids,
            "vision_context".to_string(),
            tick,
        );
        binding_clump.coherence = 1.0;
        Self { edge_pulses, color_pulses, motion_pulses, binding_clump }
    }

    /// Total number of pulses in the bundle.
    pub fn total_pulses(&self) -> usize {
        self.edge_pulses.len() + self.color_pulses.len() + self.motion_pulses.len()
    }

    /// Flatten stage pulses into a vector for engine injection.
    pub fn into_pulses(self) -> Vec<Pulse> {
        let mut all = self.edge_pulses;
        all.extend(self.color_pulses);
        all.extend(self.motion_pulses);
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bionetic_core::pulse::Pulse;

    fn sample(id: u64) -> Pulse {
        Pulse::value_pulse(DotId(id), 0.5, 0.5, 0)
    }

    #[test]
    fn test_bind_creates_clump() {
        let bundle = ContextBundle::bind(vec![sample(1), sample(2)], vec![sample(3)], vec![sample(4)], 100);
        assert_eq!(bundle.total_pulses(), 4);
        assert_eq!(bundle.binding_clump.member_dots.len(), 4);
        assert_eq!(bundle.binding_clump.formed_tick, 100);
    }

    #[test]
    fn test_duplicate_dots_deduped() {
        let bundle = ContextBundle::bind(vec![sample(5)], vec![sample(5)], Vec::new(), 0);
        assert_eq!(bundle.binding_clump.member_dots.len(), 1);
    }
}
