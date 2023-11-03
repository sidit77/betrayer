use crate::error::{ErrorSource, TrayResult};
use crate::{Menu, TrayEvent, TrayIconBuilder};

pub struct NativeTrayIcon<T> {
    signal: Vec<T>
}

impl<T: Clone + 'static> NativeTrayIcon<T> {

    pub fn new<F>(builder: TrayIconBuilder<T>, mut callback: F) -> TrayResult<Self>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        todo!()

    }

}

impl<T> NativeTrayIcon<T> {
    pub fn set_tooltip(&self, _tooltip: Option<String>) {

    }

}

impl<T: 'static> NativeTrayIcon<T> {
    pub fn set_menu(&self, _menu: Option<Menu<T>>) {

    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NativeIcon;

impl NativeIcon {
    pub fn from_rgba(_rgba: Vec<u8>, _width: u32, _height: u32) -> TrayResult<Self> {
        Ok(Self)
    }
}


pub type PlatformError = ();
impl From<PlatformError> for ErrorSource {
    fn from(value: PlatformError) -> Self {
        ErrorSource::Os(value)
    }
}