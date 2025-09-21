use autd3::prelude::mm;

/// Options for RMS recording.
#[derive(Debug, Clone, Copy)]
pub struct RmsRecordOption {
    /// Sound speed [mm/s].
    pub sound_speed: f32,
    #[cfg_attr(docsrs, doc(cfg(feature = "remote")))]
    #[cfg(feature = "gpu")]
    /// If true, use GPU for computation.
    pub gpu: bool,
}

impl std::default::Default for RmsRecordOption {
    fn default() -> Self {
        Self {
            sound_speed: 340e3 * mm,
            #[cfg(feature = "gpu")]
            gpu: false,
        }
    }
}
