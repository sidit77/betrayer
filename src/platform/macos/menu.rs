use icrate::AppKit::{NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuItem};
use icrate::Foundation::{MainThreadMarker, NSString};
use objc2::ffi::NSInteger;
use objc2::rc::Id;
use crate::{Menu, MenuItem};
use crate::platform::macos::callback::SystemTrayCallback;

pub unsafe fn build_menu_item<T>(marker: MainThreadMarker, item: MenuItem<T>, callback: &SystemTrayCallback, signal_map: &mut Vec<T>) -> Id<NSMenuItem> {
    match item {
        MenuItem::Separator => NSMenuItem::separatorItem(marker),
        MenuItem::Button { name, checked, signal } => {
            let button = NSMenuItem::new(marker);
            button.setTitle(&NSString::from_str(&name));
            //let button = NSMenuItem::initWithTitle_action_keyEquivalent(
            //    NSMenuItem::new(marker),
            //    &NSString::from_str(&name),
            //    None,
            //    &NSString::from_str("")
            //);
            button.setState(match checked.unwrap_or_default() {
                true => NSControlStateValueOn,
                false => NSControlStateValueOff
            });
            button.setTarget(Some(callback));
            button.setAction(Some(SystemTrayCallback::selector()));
            button.setTag(signal_map.len() as NSInteger);
            signal_map.push(signal);
            button
        },
        MenuItem::Menu { name, children } => {
            let sub = NSMenu::new(marker);
            for item in children {
                sub.addItem(&build_menu_item(marker, item, callback, signal_map));
            }
            let button = NSMenuItem::new(marker);
            button.setTitle(&NSString::from_str(&name));
            //let button = NSMenuItem::initWithTitle_action_keyEquivalent(
            //    NSMenuItem::alloc(),
            //    &NSString::from_str(&name),
            //    None,
            //    &NSString::from_str("")
            //);
            button.setSubmenu(Some(&sub));
            button
        }
    }
}

pub fn construct_native_menu<T>(marker: MainThreadMarker, menu: Menu<T>, callback: &SystemTrayCallback) -> (Id<NSMenu>, Vec<T>) {
    unsafe {
        let mut signal_map = Vec::new();
        let native_menu = NSMenu::new(marker);
        for item in menu.items {
            native_menu.addItem(&build_menu_item(marker,item, callback, &mut signal_map));
        }
        (native_menu, signal_map)
    }

}