//! Wall-clock abstraction. Local 12-hour format is the default; 24-hour is available.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockFormat {
    H12,
    H24,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub is_pm: bool,
    pub format: ClockFormat,
    pub epoch_secs: i64,
}

impl WallTime {
    pub fn hour_24(&self) -> u8 {
        match self.format {
            ClockFormat::H24 => self.hour,
            ClockFormat::H12 => {
                let hour = if self.hour == 12 { 0 } else { self.hour };
                if self.is_pm { hour + 12 } else { hour }
            }
        }
    }
}

pub trait WallClock: Send + Sync {
    fn now(&self) -> WallTime;
    fn format(&self) -> ClockFormat;
}

pub struct SystemWallClock {
    prefer: ClockFormat,
}

impl SystemWallClock {
    pub fn new() -> Self { Self { prefer: ClockFormat::H12 } }
    pub fn with_format(prefer: ClockFormat) -> Self { Self { prefer } }
}

impl Default for SystemWallClock {
    fn default() -> Self { Self::new() }
}

impl WallClock for SystemWallClock {
    fn now(&self) -> WallTime {
        use chrono::{Local, Timelike};
        let epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or(0);
        let local = Local::now();
        let hour_24 = local.hour() as u8;
        let minute = local.minute() as u8;
        let second = local.second() as u8;
        match self.prefer {
            ClockFormat::H24 => WallTime {
                hour: hour_24, minute, second, is_pm: hour_24 >= 12,
                format: ClockFormat::H24, epoch_secs,
            },
            ClockFormat::H12 => WallTime {
                hour: match hour_24 % 12 { 0 => 12, hour => hour },
                minute,
                second,
                is_pm: hour_24 >= 12,
                format: ClockFormat::H12,
                epoch_secs,
            },
        }
    }

    fn format(&self) -> ClockFormat { self.prefer }
}

pub struct FakeWallClock {
    epoch_secs: AtomicI64,
    format: ClockFormat,
}

impl FakeWallClock {
    pub fn new(epoch_secs: i64, format: ClockFormat) -> Self {
        Self { epoch_secs: AtomicI64::new(epoch_secs), format }
    }

    pub fn advance_secs(&self, seconds: i64) {
        self.epoch_secs.fetch_add(seconds, Ordering::SeqCst);
    }

    pub fn set(&self, epoch_secs: i64) {
        self.epoch_secs.store(epoch_secs, Ordering::SeqCst);
    }
}

impl WallClock for FakeWallClock {
    fn now(&self) -> WallTime {
        let epoch_secs = self.epoch_secs.load(Ordering::SeqCst);
        let day_secs = epoch_secs.rem_euclid(86_400) as u32;
        let hour_24 = (day_secs / 3600) as u8;
        let minute = ((day_secs % 3600) / 60) as u8;
        let second = (day_secs % 60) as u8;
        match self.format {
            ClockFormat::H24 => WallTime {
                hour: hour_24, minute, second, is_pm: hour_24 >= 12,
                format: ClockFormat::H24, epoch_secs,
            },
            ClockFormat::H12 => WallTime {
                hour: match hour_24 % 12 { 0 => 12, hour => hour },
                minute,
                second,
                is_pm: hour_24 >= 12,
                format: ClockFormat::H12,
                epoch_secs,
            },
        }
    }

    fn format(&self) -> ClockFormat { self.format }
}
