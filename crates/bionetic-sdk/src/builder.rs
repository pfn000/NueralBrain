//! Builder for constructing a brain with validated configuration.

use crate::error::{SdkError, SdkResult};
use crate::handle::BrainHandle;
use bionetic_engine::engine::BioneticEngine;
use serde::{Deserialize, Serialize};

/// Fully-specified brain configuration — the output of the builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    /// Number of corpora (0 .. num_corpora).
    pub num_corpora: usize,
    /// Initial dots per corpus (before neurogenesis).
    pub dots_per_corpus: usize,
    /// Enable background dream-state consolidation.
    pub enable_dreaming: bool,
    /// Enable adaptive neurogenesis.
    pub enable_neurogenesis: bool,
    /// Simulation tick rate in Hz.
    pub tick_rate_hz: u32,
    /// Max pulses injected per `think()` call (safety limit).
    pub max_input_pulses: usize,
    /// Enable detailed event logging.
    pub verbose: bool,
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            num_corpora: bionetic_core::CORPUS_COUNT_TOTAL,
            dots_per_corpus: 128,
            enable_dreaming: true,
            enable_neurogenesis: true,
            tick_rate_hz: 100,
            max_input_pulses: 4096,
            verbose: false,
        }
    }
}

impl BrainConfig {
    /// Validate the configuration. Called by the builder on `.build()`.
    pub fn validate(&self) -> SdkResult<()> {
        if self.num_corpora == 0 {
            return Err(SdkError::InvalidConfig(
                "num_corpora must be ≥ 1".into(),
            ));
        }
        if self.num_corpora > 256 {
            return Err(SdkError::InvalidConfig(
                "num_corpora must be ≤ 256".into(),
            ));
        }
        if self.dots_per_corpus > 4096 {
            return Err(SdkError::InvalidConfig(
                "dots_per_corpus must be ≤ 4096".into(),
            ));
        }
        if self.dots_per_corpus == 0 {
            return Err(SdkError::InvalidConfig(
                "dots_per_corpus must be ≥ 1".into(),
            ));
        }
        if self.tick_rate_hz == 0 {
            return Err(SdkError::InvalidConfig(
                "tick_rate_hz must be ≥ 1".into(),
            ));
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────
// Builder
// ──────────────────────────────────────────────

/// Fluent builder for creating a `BrainHandle`.
///
/// # Example
/// ```
/// use bionetic_sdk::BrainBuilder;
///
/// let brain = BrainBuilder::new()
///     .with_corpora(bionetic_core::CORPUS_COUNT_TOTAL)
///     .with_dots_per_corpus(128)
///     .with_dreaming(true)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct BrainBuilder {
    config: BrainConfig,
}

impl BrainBuilder {
    /// Start a new builder with default settings.
    pub fn new() -> Self {
        Self {
            config: BrainConfig::default(),
        }
    }

    /// Start from an existing configuration (useful for config files).
    pub fn from_config(config: BrainConfig) -> Self {
        Self { config }
    }

    // ── Fluent setters ──

    /// Set the number of corpora (default: 14).
    pub fn with_corpora(mut self, n: usize) -> Self {
        self.config.num_corpora = n;
        self
    }

    /// Set the initial dots per corpus (default: 128).
    pub fn with_dots_per_corpus(mut self, n: usize) -> Self {
        self.config.dots_per_corpus = n;
        self
    }

    /// Enable or disable background dreaming (default: true).
    pub fn with_dreaming(mut self, yes: bool) -> Self {
        self.config.enable_dreaming = yes;
        self
    }

    /// Enable or disable adaptive neurogenesis (default: true).
    pub fn with_neurogenesis(mut self, yes: bool) -> Self {
        self.config.enable_neurogenesis = yes;
        self
    }

    /// Set the tick rate in Hz (default: 100).
    pub fn with_tick_rate(mut self, hz: u32) -> Self {
        self.config.tick_rate_hz = hz;
        self
    }

    /// Set the max input pulses per `think()` call (default: 4096).
    pub fn with_max_input_pulses(mut self, n: usize) -> Self {
        self.config.max_input_pulses = n;
        self
    }

    /// Enable verbose event logging (default: false).
    pub fn with_verbose(mut self, yes: bool) -> Self {
        self.config.verbose = yes;
        self
    }

    /// Replace the entire configuration.
    pub fn with_config(mut self, config: BrainConfig) -> Self {
        self.config = config;
        self
    }

    // ── Construction ──

    /// Build the brain. Panics if configuration is invalid.
    /// Use `try_build()` for a non-panicking version.
    pub fn build(self) -> BrainHandle {
        self.try_build().expect("invalid brain configuration")
    }

    /// Build the brain, returning an error if configuration is invalid.
    pub fn try_build(self) -> SdkResult<BrainHandle> {
        self.config.validate()?;
        let engine = BioneticEngine::with_config(
            self.config.num_corpora,
            self.config.dots_per_corpus,
            self.config.enable_neurogenesis,
        ).map_err(SdkError::InvalidConfig)?;
        Ok(BrainHandle::from_parts(engine, self.config))
    }

    /// Access the current config (for inspection before build).
    pub fn config(&self) -> &BrainConfig {
        &self.config
    }
}

impl Default for BrainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_builder() {
        let b = BrainBuilder::new();
        assert_eq!(b.config().num_corpora, bionetic_core::CORPUS_COUNT_TOTAL);
        assert_eq!(b.config().dots_per_corpus, 128);
        assert!(b.config().enable_dreaming);
    }

    #[test]
    fn test_fluent_setters() {
        let b = BrainBuilder::new()
            .with_corpora(16)
            .with_dots_per_corpus(256)
            .with_dreaming(false)
            .with_tick_rate(60);

        assert_eq!(b.config().num_corpora, 16);
        assert_eq!(b.config().dots_per_corpus, 256);
        assert!(!b.config().enable_dreaming);
        assert_eq!(b.config().tick_rate_hz, 60);
    }

    #[test]
    fn test_validation_zero_corpora() {
        let result = BrainBuilder::new().with_corpora(0).try_build();
        assert!(matches!(result, Err(SdkError::InvalidConfig(_))));
    }

    #[test]
    fn test_validation_too_many_corpora() {
        let result = BrainBuilder::new().with_corpora(300).try_build();
        assert!(matches!(result, Err(SdkError::InvalidConfig(_))));
    }

    #[test]
    fn test_validation_zero_dots() {
        let result = BrainBuilder::new().with_dots_per_corpus(0).try_build();
        assert!(matches!(result, Err(SdkError::InvalidConfig(_))));
    }

    #[test]
    fn test_successful_build() {
        let brain = BrainBuilder::new()
            .with_corpora(4)
            .with_dots_per_corpus(32)
            .try_build();
        assert!(brain.is_ok());
    }

    #[test]
    fn test_config_roundtrip() {
        let config = BrainConfig {
            num_corpora: 12,
            dots_per_corpus: 100,
            enable_dreaming: false,
            enable_neurogenesis: true,
            tick_rate_hz: 200,
            max_input_pulses: 2048,
            verbose: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let restored: BrainConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.num_corpora, 12);
        assert_eq!(restored.tick_rate_hz, 200);
    }
}