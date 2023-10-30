use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::panic::Location;

use crate::platform::PlatformError;

pub type TrayResult<T> = Result<T, TrayError>;

#[derive(Debug)]
pub enum ErrorSource {
    Os(PlatformError),
    Custom(Cow<'static, str>)
}

pub struct TrayError {
    location: &'static Location<'static>,
    source: ErrorSource
}

impl TrayError {
    #[track_caller]
    pub fn custom(msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            location: Location::caller(),
            source: ErrorSource::Custom(msg.into())
        }
    }

}

impl Debug for TrayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HidError: {:?}\n\tat {}", self.source, self.location)
    }
}

impl Display for TrayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.source)
    }
}

impl Error for TrayError {}

impl<T: Into<ErrorSource>> From<T> for TrayError {
    #[track_caller]
    fn from(value: T) -> Self {
        Self {
            location: Location::caller(),
            source: value.into()
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $result:expr) => {
        if !($cond) {
            return Err($result);
        }
    };
}