mod menu;
mod callback;

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;
use icrate::AppKit::{NSApplication, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use icrate::Foundation::NSString;
use objc2::rc::Id;
use crate::error::TrayResult;
use crate::{Menu, TrayEvent, TrayIconBuilder};
use crate::platform::macos::callback::SystemTrayCallback;
use crate::platform::macos::menu::construct_native_menu;
use crate::utils::OptionCellExt;

pub struct NativeTrayIcon<T> {
    status_item: Id<NSStatusItem>,
    signal_map: Rc<Cell<Option<Vec<T>>>>,
    callback: Id<SystemTrayCallback>,
    _marker: PhantomData<T>
}

impl<T: Clone + 'static> NativeTrayIcon<T> {
    pub fn new<F>(builder: TrayIconBuilder<T>, callback: F) -> TrayResult<Self>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        unsafe {
            NSApplication::sharedApplication();

            let status_bar = NSStatusBar::systemStatusBar();
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            let signal_map = Rc::new(Cell::new(None));

            let callback = {
                let signal_map = signal_map.clone();
                let callback = RefCell::new(callback);
                SystemTrayCallback::new(move |tag| {
                    let signal = signal_map
                        .with(|map: &mut Vec<T> | map.get(tag as usize).cloned())
                        .flatten();
                    if let Some(signal) = signal {
                        callback.borrow_mut()(TrayEvent::Menu(signal));
                    } else {
                        log::debug!("Failed to get signal for tag {}", tag);
                    }
                })
            };

            if let Some(button) = status_item.button() {
                button.setTitle(&NSString::from_str("TEST BUTTON"));
            }

            if let Some((menu, map)) = builder.menu.map(|menu| construct_native_menu(menu, &callback)) {
                status_item.setMenu(Some(&menu));
                signal_map.set(Some(map));
            }

            Ok(Self {
                status_item,
                signal_map,
                callback,
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