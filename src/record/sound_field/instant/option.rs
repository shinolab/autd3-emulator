use std::time::Duration;

use autd3::prelude::mm;

/// Options for instant recording.
#[derive(Debug, Clone, Copy)]
pub struct InstantRecordOption {
    /// Sound speed \[mm/s\].
    pub sound_speed: f32,
    /// Time step.
    pub time_step: Duration,
    /// Memory limits hint \[MB\].
    pub memory_limits_hint_mb: usize,
    #[cfg(feature = "gpu")]
    /// If true, use GPU for computation.
    pub gpu: bool,
}

impl std::default::Default for InstantRecordOption {
    fn default() -> Self {
        Self {
            sound_speed: 340e3 * mm,
            time_step: Duration::from_micros(1),
            memory_limits_hint_mb: 128,
            #[cfg(feature = "gpu")]
            gpu: false,
        }
    }
}
