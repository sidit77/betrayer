use std::ptr::NonNull;

use block2::{Block, ConcreteBlock, RcBlock};
use icrate::AppKit::NSMenuItem;
use objc2::{ClassType, declare_class, msg_send_id, msg_send, sel};
use objc2::runtime::{NSObject, Sel};
use objc2::declare::{Ivar, IvarDrop};
use objc2::ffi::NSInteger;
use objc2::mutability::InteriorMutable;
use objc2::rc::Id;

declare_class!(
    #[derive(Debug)]
    pub struct SystemTrayCallback {
        callback: IvarDrop<Box<RcBlock<(NSInteger,), ()>>, "_callback">,
    }

    mod ivars;

    unsafe impl ClassType for SystemTrayCallback {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "SystemTrayCallback";
    }

    unsafe impl SystemTrayCallback {
        #[method(initWithCallback:)]
        unsafe fn init(this: *mut Self, callback: *mut Block<(NSInteger,), ()>) -> Option<NonNull<Self>> {
            let this: Option<&mut Self> = msg_send![super(this), init];
            let Some(this) = this else {
                return None;
            };

            Ivar::write(&mut this.callback, Box::new(RcBlock::copy(callback)));

            Some(NonNull::from(this))
        }

        #[method(call_menu_item:)]
        unsafe fn call_menu_item(&self, sender: *mut NSMenuItem) {
            if let Some(sender) = sender.as_ref() {
                self.callback.call((sender.tag(),));
            }
        }
    }
);

impl SystemTrayCallback {
    fn from_block(callback: &Block<(NSInteger,), ()>) -> Id<Self> {
        unsafe { msg_send_id![Self::alloc(), initWithCallback: callback] }
    }

    pub fn new<F: Fn(NSInteger) + 'static>(callback: F) -> Id<Self> {
        let callback_block = ConcreteBlock::new(callback).copy();
        Self::from_block(&*callback_block)
    }

    pub fn menu_item_selector() -> Sel {
        sel!(call_menu_item:)
    }
}
