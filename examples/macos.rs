use std::sync::mpsc::channel;
use std::ptr::NonNull;
use block2::{Block, ConcreteBlock, RcBlock};
use icrate::AppKit::{NSApplication, NSMenu, NSMenuItem, NSStatusBar, NSVariableStatusItemLength};
use icrate::Foundation::NSString;
use objc2::{ClassType, declare_class, msg_send_id, msg_send, sel};
use objc2::declare::{IvarDrop, Ivar};
use objc2::mutability::InteriorMutable;
use objc2::rc::{Id};
use objc2::runtime::NSObject;
use system_status_bar_macos::sync_infinite_event_loop;

fn main() {
    unsafe {
        NSApplication::sharedApplication();

        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

        if let Some(button) = status_item.button() {
            button.setTitle(&NSString::from_str("TEST BUTTON"));
        }


        let callback = {
            let callback_block = ConcreteBlock::new(move |item: *mut NSMenuItem| {
                let id = (*item).tag();
                println!("Click: {}", id);
            }).copy();
            STBMenuItemCallback::new(&*callback_block)
        };

        {


            let menu_item1 = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(),
                &NSString::from_str("TEST ITEM"),
                None,
                &NSString::from_str("")
            );
            menu_item1.setTag(1);
            menu_item1.setTarget(Some(&callback));
            menu_item1.setAction(Some(sel!(call:)));

            let menu_item2 = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(),
                &NSString::from_str("TEST ITEM"),
                None,
                &NSString::from_str("")
            );
            menu_item2.setTag(2);
            menu_item2.setTarget(Some(&callback));
            menu_item2.setAction(Some(sel!(call:)));

            let menu = NSMenu::new();
            menu.addItem(&menu_item1);
            menu.addItem(&menu_item2);
            status_item.setMenu(Some(&menu));
        }

        let (_sender, receiver) = channel::<()>();
        sync_infinite_event_loop(receiver, |_| { });

        //menu.removeAllItems();
        //status_item.setMenu(None);
        status_bar.removeStatusItem(&status_item);
    }


}

declare_class!(
    #[derive(Debug)]
    struct STBMenuItemCallback {
        callback: IvarDrop<Box<RcBlock<(*mut NSMenuItem,), ()>>, "_callback">,
    }

    mod ivars;

    unsafe impl ClassType for STBMenuItemCallback {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "STBMenuItemCallback";
    }

    unsafe impl STBMenuItemCallback {
        #[method(initWithCallback:)]
        unsafe fn init(this: *mut Self, callback: *mut Block<(*mut NSMenuItem,), ()>) -> Option<NonNull<Self>> {
            let this: Option<&mut Self> = msg_send![super(this), init];
            let Some(this) = this else {
                return None;
            };

            Ivar::write(&mut this.callback, Box::new(RcBlock::copy(callback)));

            Some(NonNull::from(this))
        }

        #[method(call:)]
        unsafe fn call(&self, sender: *mut NSMenuItem) {
            self.callback.call((sender,));
        }
    }
);

impl STBMenuItemCallback {
    fn new(callback: &Block<(*mut NSMenuItem,), ()>) -> Id<Self> {
        unsafe { msg_send_id![Self::alloc(), initWithCallback: callback] }
    }
}

#[derive(Debug)]
struct MenuItemCallback {
    inner: Id<STBMenuItemCallback>,
}

impl MenuItemCallback {
    fn new(callback: Box<dyn Fn() + 'static>) -> Self {
        let callback_block = ConcreteBlock::new(move |_: *mut NSMenuItem| {
            callback();
        }).copy();
        let inner = STBMenuItemCallback::new(&*callback_block);
        Self { inner }
    }
}