mod range;
mod time_range;

use autd3_driver::defined::mm;
pub use range::Range;
pub use time_range::TimeRange;

pub struct RecordOption {
    pub sound_speed: f32,
    pub time: Option<TimeRange>,
    pub print_progress: bool,
}

impl std::default::Default for RecordOption {
    fn default() -> Self {
        Self {
            sound_speed: 340e3 * mm,
            time: None,
            print_progress: false,
        }
    }
}

impl RecordOption {
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
}
