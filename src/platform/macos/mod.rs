mod menu;

use std::marker::PhantomData;
use icrate::AppKit::{NSApplication, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use icrate::Foundation::NSString;
use objc2::rc::Id;
use crate::error::TrayResult;
use crate::{Menu, TrayEvent, TrayIconBuilder};
use crate::platform::macos::menu::construct_native_menu;

pub struct NativeTrayIcon<T> {
    status_item: Id<NSStatusItem>,
    _marker: PhantomData<T>
}

impl<T: Clone + 'static> NativeTrayIcon<T> {
    pub fn new<F>(builder: TrayIconBuilder<T>, _callback: F) -> TrayResult<Self>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        unsafe {
            NSApplication::sharedApplication();

            let status_bar = NSStatusBar::systemStatusBar();
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            if let Some(button) = status_item.button() {
                button.setTitle(&NSString::from_str("TEST BUTTON"));
            }

            if let Some(menu) = builder.menu.map(construct_native_menu) {
                status_item.setMenu(Some(&menu));
            }

            Ok(Self {
                status_item,
                _marker: Default::default(),
            })
        }
    }
}

impl<T> Drop for NativeTrayIcon<T> {
    fn drop(&mut self) {
        unsafe {
            let status_bar = self
                .status_item
                .statusBar()
                .unwrap_or_else(|| NSStatusBar::systemStatusBar());
            status_bar.removeStatusItem(&self.status_item);
        }
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
        Ok(NativeIcon)
    }
}

pub type PlatformError = ();