use autd3::prelude::mm;

/// Options for RMS recording.
#[derive(Debug, Clone, Copy)]
pub struct RmsRecordOption {
    /// Sound speed [mm/s].
    pub sound_speed: f32,
    /// If true, print progress bar.
    pub print_progress: bool,
    #[cfg_attr(docsrs, doc(cfg(feature = "remote")))]
    #[cfg(feature = "gpu")]
    /// If true, use GPU for computation.
    pub gpu: bool,
}

impl std::default::Default for RmsRecordOption {
    fn default() -> Self {
        Self {
            sound_speed: 340e3 * mm,
            print_progress: false,
            #[cfg(feature = "gpu")]
            gpu: false,
        }
    }
}

impl RmsRecordOption {
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
