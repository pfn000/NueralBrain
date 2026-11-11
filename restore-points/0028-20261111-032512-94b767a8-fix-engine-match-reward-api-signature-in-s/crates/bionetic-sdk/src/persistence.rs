//! Brain state persistence — save and load to bytes or files.

use crate::builder::BrainConfig;
use crate::error::{SdkError, SdkResult};
use serde::{Deserialize, Serialize};

/// Serialization format for saved state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaveFormat {
    /// Human-readable JSON (debug-friendly, large).
    Json,
    /// Pretty-printed JSON (bigger, easier to inspect).
    JsonPretty,
}

impl Default for SaveFormat {
    fn default() -> Self {
        SaveFormat::Json
    }
}

/// A versioned bundle containing all state needed to restore a brain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBundle {
    /// Bundle format version.
    pub version: u16,
    /// The brain's configuration at save time.
    pub config: BrainConfig,
    /// Engine state blob (opaque to the SDK — handled by bionetic-engine).
    pub engine_state: serde_json::Value,
    /// Tick at which the brain was saved.
    pub saved_at_tick: u64,
}

impl StateBundle {
    /// Current bundle format version.
    pub const CURRENT_VERSION: u16 = 1;

    /// Encode this bundle as bytes.
    pub fn to_bytes(&self, format: SaveFormat) -> SdkResult<Vec<u8>> {
        match format {
            SaveFormat::Json => Ok(serde_json::to_vec(self)?),
            SaveFormat::JsonPretty => Ok(serde_json::to_vec_pretty(self)?),
        }
    }

    /// Decode bytes into a bundle.
    pub fn from_bytes(bytes: &[u8]) -> SdkResult<Self> {
        let bundle: StateBundle = serde_json::from_slice(bytes)?;
        if bundle.version > Self::CURRENT_VERSION {
            return Err(SdkError::Persistence(format!(
                "state bundle version {} newer than SDK version {}",
                bundle.version,
                Self::CURRENT_VERSION
            )));
        }
        Ok(bundle)
    }

    /// Save to a file.
    #[cfg(feature = "native")]
    pub fn save_to_file(&self, path: &std::path::Path, format: SaveFormat) -> SdkResult<()> {
        let bytes = self.to_bytes(format)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load from a file.
    #[cfg(feature = "native")]
    pub fn load_from_file(path: &std::path::Path) -> SdkResult<Self> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_roundtrip() {
        let bundle = StateBundle {
            version: StateBundle::CURRENT_VERSION,
            config: BrainConfig::default(),
            engine_state: serde_json::json!({ "dots": 512 }),
            saved_at_tick: 1000,
        };

        let bytes = bundle.to_bytes(SaveFormat::Json).unwrap();
        let restored = StateBundle::from_bytes(&bytes).unwrap();
        assert_eq!(restored.saved_at_tick, 1000);
        assert_eq!(restored.version, 1);
    }

    #[test]
    fn test_pretty_format() {
        let bundle = StateBundle {
            version: 1,
            config: BrainConfig::default(),
            engine_state: serde_json::json!({}),
            saved_at_tick: 0,
        };
        let pretty = bundle.to_bytes(SaveFormat::JsonPretty).unwrap();
        let compact = bundle.to_bytes(SaveFormat::Json).unwrap();
        assert!(pretty.len() > compact.len());
    }

    #[test]
    fn test_future_version_rejected() {
        let future = StateBundle {
            version: 999,
            config: BrainConfig::default(),
            engine_state: serde_json::json!({}),
            saved_at_tick: 0,
        };
        let bytes = future.to_bytes(SaveFormat::Json).unwrap();
        let result = StateBundle::from_bytes(&bytes);
        assert!(matches!(result, Err(SdkError::Persistence(_))));
    }
}