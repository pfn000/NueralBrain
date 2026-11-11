//! Brain state serialization for persistence (IndexedDB, file, etc.)

use serde_json;
use bionetic_engine::BioneticEngine;

/// Serialize brain status to JSON string
pub fn serialize_status(engine: &BioneticEngine) -> String {
    serde_json::json!({
        "tick": engine.tick,
        "state": format!("{:?}", engine.state),
        "total_dots": engine.total_dots(),
        "total_connections": engine.total_connections(),
        "total_clumps": engine.total_clumps(),
        "pending_pulses": engine.pending_pulses(),
    }).to_string()
}
