use std::ptr::NonNull;

use block2::{Block, ConcreteBlock, RcBlock};
use icrate::AppKit::NSMenuItem;
use objc2::{ClassType, declare_class, msg_send_id, msg_send, sel};
use objc2::runtime::{NSObject, Sel};
use objc2::declare::{Ivar, IvarDrop};
use objc2::mutability::InteriorMutable;
use objc2::rc::Id;

declare_class!(
    #[derive(Debug)]
    pub struct SystemTrayCallback {
        callback: IvarDrop<Box<RcBlock<(*mut NSMenuItem,), ()>>, "_callback">,
    }

    mod ivars;

    unsafe impl ClassType for SystemTrayCallback {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "SystemTrayCallback";
    }

    unsafe impl SystemTrayCallback {
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

impl SystemTrayCallback {
    fn new(callback: &Block<(*mut NSMenuItem,), ()>) -> Id<Self> {
        unsafe { msg_send_id![Self::alloc(), initWithCallback: callback] }
    }

    pub fn new2<F: Fn() + 'static>(callback: F) -> Id<Self> {
        let callback_block = ConcreteBlock::new(move |item: *mut NSMenuItem| {
            callback();
        }).copy();
        Self::new(&*callback_block)
    }

    pub fn menu_item_selector() -> Sel {
        sel!(call:)
    }
}
