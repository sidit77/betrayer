use icrate::AppKit::{NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuItem};
use icrate::Foundation::NSString;
use objc2::ClassType;
use objc2::ffi::NSInteger;
use objc2::rc::Id;
use crate::{Menu, MenuItem};
use crate::platform::macos::callback::SystemTrayCallback;

pub unsafe fn build_menu_item<T>(item: MenuItem<T>, callback: &SystemTrayCallback, signal_map: &mut Vec<T>) -> Id<NSMenuItem> {
    match item {
        MenuItem::Separator => NSMenuItem::separatorItem(),
        MenuItem::Button { name, checked, signal } => {
            let button = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(),
                &NSString::from_str(&name),
                None,
                &NSString::from_str("")
            );
            button.setState(match checked {
                true => NSControlStateValueOn,
                false => NSControlStateValueOff
            });
            button.setTarget(Some(callback));
            button.setAction(Some(SystemTrayCallback::menu_item_selector()));
            button.setTag(signal_map.len() as NSInteger);
            signal_map.push(signal);
            button
        },
        MenuItem::Menu { name, children } => {
            let sub = NSMenu::new();
            for item in children {
                sub.addItem(&build_menu_item(item, callback, signal_map));
            }
            let button = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(),
                &NSString::from_str(&name),
                None,
                &NSString::from_str("")
            );
            button.setSubmenu(Some(&sub));
            button
        }
    }
}

pub fn construct_native_menu<T>(menu: Menu<T>, callback: &SystemTrayCallback) -> (Id<NSMenu>, Vec<T>) {
    unsafe {
        let mut signal_map = Vec::new();
        let native_menu = NSMenu::new();
        for item in menu.items {
            native_menu.addItem(&build_menu_item(item, callback, &mut signal_map));
        }
        (native_menu, signal_map)
    }

}