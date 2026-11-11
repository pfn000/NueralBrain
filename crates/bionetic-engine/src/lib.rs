//! # Bionetic Engine
//!
//! The processing heartbeat of the bionetic brain.
//! Manages tick cycles, pulse delivery, clump management,
//! plasticity, memory consolidation, and sleep states.

pub mod engine;
pub mod processor;
pub mod plasticity;
pub mod memory;
pub mod hippocampus;
pub mod reward;
pub mod dream;
pub mod sleep;

pub use engine::BioneticEngine;
pub mod brain_protocol;

pub mod region_controller;
pub mod wall_clock;
pub mod schedule;
pub mod maintenance;
pub use region_controller::{RegionController, RegionTransition, RegionEvent};
pub use wall_clock::{WallClock, SystemWallClock, FakeWallClock, ClockFormat, WallTime};
pub use schedule::{SleepSchedule, SleepScheduleConfig, SchedulePhase};
pub use maintenance::{MaintenancePass, MaintenanceReport, MaintenanceConfig};
