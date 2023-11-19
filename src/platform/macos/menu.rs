use icrate::AppKit::{NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuItem};
use icrate::Foundation::NSString;
use objc2::ClassType;
use objc2::rc::Id;
use crate::{Menu, MenuItem};

pub unsafe fn build_menu_item<T>(item: MenuItem<T>) -> Id<NSMenuItem> {
    match item {
        MenuItem::Separator => NSMenuItem::separatorItem(),
        MenuItem::Button { name, checked, .. } => {
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
            button
        },
        MenuItem::Menu { name, children } => {
            let sub = NSMenu::new();
            for item in children {
                sub.addItem(&build_menu_item(item));
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

pub fn construct_native_menu<T>(menu: Menu<T>) -> Id<NSMenu> {
    unsafe {
        let m = NSMenu::new();
        for item in menu.items {
            m.addItem(&build_menu_item(item));
        }
        m
    }

}