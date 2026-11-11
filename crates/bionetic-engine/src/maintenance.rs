//! Bounded, round-robin maintenance selection across sleep-eligible regions.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use bionetic_core::{CorpusId, RegionState};
use crate::region_controller::RegionController;

#[derive(Debug, Clone)]
pub struct MaintenanceConfig {
    pub repair_enabled: bool,
    pub enhance_enabled: bool,
    pub prune_enabled: bool,
    pub enhance_delta: f64,
    pub prune_threshold: f64,
    pub prune_window_ticks: u64,
    pub max_corpora_per_pass: usize,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            repair_enabled: true,
            enhance_enabled: true,
            prune_enabled: true,
            enhance_delta: 0.01,
            prune_threshold: 0.02,
            prune_window_ticks: 10_000,
            max_corpora_per_pass: 4,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaintenanceReport {
    pub corpora_processed: usize,
    pub dots_repaired: usize,
    pub connections_enhanced: usize,
    pub connections_pruned: usize,
    pub elapsed_us: u64,
}

pub struct MaintenancePass {
    config: MaintenanceConfig,
    controller: std::sync::Arc<RegionController>,
    cursor: AtomicUsize,
}

impl MaintenancePass {
    pub fn new(controller: std::sync::Arc<RegionController>, config: MaintenanceConfig) -> Self {
        Self { config, controller, cursor: AtomicUsize::new(0) }
    }

    pub fn config(&self) -> &MaintenanceConfig { &self.config }

    pub fn run<F>(&self, mut per_corpus: F) -> MaintenanceReport
    where
        F: FnMut(CorpusId, &MaintenanceConfig) -> (usize, usize, usize),
    {
        let started = Instant::now();
        let eligible: Vec<CorpusId> = self.controller.all_statuses().into_iter()
            .filter(|status| matches!(
                status.state,
                RegionState::Resting | RegionState::Dormant | RegionState::Enhancing
            ))
            .map(|status| status.corpus)
            .collect();
        let mut report = MaintenanceReport::default();
        if eligible.is_empty() || self.config.max_corpora_per_pass == 0 {
            report.elapsed_us = started.elapsed().as_micros() as u64;
            return report;
        }

        let count = self.config.max_corpora_per_pass.min(eligible.len());
        let offset = self.cursor.fetch_add(count, Ordering::Relaxed);
        for index in 0..count {
            let corpus = eligible[(offset + index) % eligible.len()];
            let (repaired, enhanced, pruned) = per_corpus(corpus, &self.config);
            report.corpora_processed += 1;
            report.dots_repaired += repaired;
            report.connections_enhanced += enhanced;
            report.connections_pruned += pruned;
        }
        report.elapsed_us = started.elapsed().as_micros() as u64;
        report
    }
}
