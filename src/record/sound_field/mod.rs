use crate::{EmulatorError, Range};

use super::Record;

pub(crate) mod instant;
pub(crate) mod rms;

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    #[autd3::driver::async_trait]
    pub trait SoundFieldOption<'a> {
        type Output;

        async fn sound_field(
            self,
            record: &'a Record,
            range: Range,
        ) -> Result<Self::Output, EmulatorError>;
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    pub trait SoundFieldOption<'a> {
        type Output;

        fn sound_field(
            self,
            record: &'a Record,
            range: Range,
        ) -> impl std::future::Future<Output = Result<Self::Output, EmulatorError>>;
    }
}

use internal::SoundFieldOption;

impl Record {
    pub async fn sound_field<'a, T: SoundFieldOption<'a>>(
        &'a self,
        range: Range,
        option: T,
    ) -> Result<T::Output, EmulatorError> {
        option.sound_field(self, range).await
    }
}
