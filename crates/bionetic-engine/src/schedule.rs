//! Sleep/wake scheduling driven by the local WallClock.

use std::sync::Arc;
use crate::wall_clock::{WallClock, WallTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulePhase {
    Awake,
    Drowsy,
    Dreaming,
    DeepSleep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SleepScheduleConfig {
    /// Awake window in 24-hour form: inclusive start, exclusive end.
    pub awake_window: (u8, u8),
    pub drowsy_minutes: u8,
    pub deep_sleep_hour_24: u8,
}

impl Default for SleepScheduleConfig {
    fn default() -> Self {
        Self { awake_window: (6, 22), drowsy_minutes: 15, deep_sleep_hour_24: 2 }
    }
}

impl SleepScheduleConfig {
    pub fn validate(&self) -> Result<(), String> {
        let (start, end) = self.awake_window;
        if start > 23 || end > 23 || self.deep_sleep_hour_24 > 23 {
            return Err("schedule hours must be within 0..=23".into());
        }
        if self.drowsy_minutes > 120 {
            return Err("drowsy_minutes must be <= 120".into());
        }
        Ok(())
    }
}

pub struct SleepSchedule {
    clock: Arc<dyn WallClock>,
    config: SleepScheduleConfig,
}

impl SleepSchedule {
    pub fn new(clock: Arc<dyn WallClock>, config: SleepScheduleConfig) -> Self {
        Self { clock, config }
    }

    pub fn clock(&self) -> &Arc<dyn WallClock> { &self.clock }
    pub fn config(&self) -> &SleepScheduleConfig { &self.config }
    pub fn now(&self) -> WallTime { self.clock.now() }

    pub fn phase(&self) -> SchedulePhase {
        let now = self.now();
        let hour = now.hour_24();
        let (start, end) = self.config.awake_window;
        let awake = if start < end {
            hour >= start && hour < end
        } else if start > end {
            hour >= start || hour < end
        } else {
            true
        };
        if awake {
            let minutes_to_sleep = minutes_until_hour(now, end);
            if minutes_to_sleep >= 0 && minutes_to_sleep <= self.config.drowsy_minutes as i32 {
                return SchedulePhase::Drowsy;
            }
            SchedulePhase::Awake
        } else if hour == self.config.deep_sleep_hour_24 {
            SchedulePhase::DeepSleep
        } else {
            SchedulePhase::Dreaming
        }
    }
}

fn minutes_until_hour(now: WallTime, target_hour: u8) -> i32 {
    let now_minutes = now.hour_24() as i32 * 60 + now.minute as i32;
    let target_minutes = target_hour as i32 * 60;
    let difference = target_minutes - now_minutes;
    if difference < 0 { difference + 24 * 60 } else { difference }
}
