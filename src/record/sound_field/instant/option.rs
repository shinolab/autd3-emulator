use std::time::Duration;

use autd3::prelude::mm;

/// Options for instant recording.
#[derive(Debug, Clone, Copy)]
pub struct InstantRecordOption {
    /// Sound speed \[mm/s\].
    pub sound_speed: f32,
    /// Time step.
    pub time_step: Duration,
    /// If true, print progress bar.
    pub print_progress: bool,
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
            print_progress: false,
            memory_limits_hint_mb: 128,
            #[cfg(feature = "gpu")]
            gpu: false,
        }
    }
}

impl InstantRecordOption {
    // GRCOV_EXCL_START
    pub(crate) fn pb(&self, n: usize) -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::new(n as _);
        if self.print_progress {
            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed}] [{bar:40.cyan/blue}] {percent}% ({eta})",
                    )
                    .unwrap()
                    .progress_chars("#-"),
            );
        } else {
            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("")
                    .unwrap(),
            );
        }
        pb
    }
    // GRCOV_EXCL_STOP
}
