use crate::{EmulatorError, Range};

use super::Record;

pub(crate) mod instant;
pub(crate) mod rms;

pub trait SoundFieldOption<'a> {
    type Output;

    fn sound_field(
        self,
        record: &'a Record,
        range: impl Range,
    ) -> impl std::future::Future<Output = Result<Self::Output, EmulatorError>>;
}

impl Record {
    /// Calculate sound field.
    pub async fn sound_field<'a, T: SoundFieldOption<'a>>(
        &'a self,
        range: impl Range,
        option: T,
    ) -> Result<T::Output, EmulatorError> {
        option.sound_field(self, range).await
    }
}
