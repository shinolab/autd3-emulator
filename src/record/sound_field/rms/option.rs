use autd3::prelude::mm;

#[derive(Debug, Clone, Copy)]
pub struct RmsRecordOption {
    pub sound_speed: f32,
    pub print_progress: bool,
    #[cfg(feature = "gpu")]
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
