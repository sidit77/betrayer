use block2::{Block, ConcreteBlock, RcBlock};
use icrate::AppKit::NSControl;
use objc2::{ClassType, DeclaredClass, declare_class, msg_send_id, sel};
use objc2::runtime::{NSObject, Sel};
use objc2::ffi::NSInteger;
use objc2::mutability::InteriorMutable;
use objc2::rc::{Id, Allocated};

declare_class!(
    pub struct SystemTrayCallback;

    unsafe impl ClassType for SystemTrayCallback {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "SystemTrayCallback";
    }

    impl DeclaredClass for SystemTrayCallback {
        type Ivars = RcBlock<(NSInteger,), ()>;
    }

    unsafe impl SystemTrayCallback {
        #[method_id(initWithCallback:)]
        fn init_with(this: Allocated<Self>, callback: *mut Block<(NSInteger,), ()>) -> Option<Id<Self>> {
            let this = this.set_ivars(unsafe { RcBlock::copy(callback) });
            unsafe { msg_send_id![super(this), init] }
        }

        #[method(call_control:)]
        unsafe fn call_control(&self, sender: *mut NSControl) {
            if let Some(sender) = sender.as_ref() {
                self.ivars().call((sender.tag(),));
            }
        }

    }

);

impl SystemTrayCallback {
    unsafe fn from_block(callback: &Block<(NSInteger,), ()>) -> Id<Self> {
        msg_send_id![Self::alloc(), initWithCallback: callback]
    }

    pub fn new<F: Fn(NSInteger) + 'static>(callback: F) -> Id<Self> {
        let callback_block = ConcreteBlock::new(callback).copy();
        unsafe { Self::from_block(&callback_block) }
    }

    pub fn selector() -> Sel {
        sel!(call_control:)
    }
}
