use std::fmt::{Debug, Formatter};
use std::ptr::null_mut;
use std::sync::Arc;

use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIcon, DestroyIcon, LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTSIZE};

use crate::error::TrayResult;
use crate::platform::windows::{error_check, get_instance_handle};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NativeIcon {
    handle: Arc<NativeIconHandle>
}

impl NativeIcon {
    pub fn from_rgba(mut rgba: Vec<u8>, width: u32, height: u32) -> TrayResult<Self> {
        let mut mask = Vec::with_capacity(rgba.len() / 4);
        let bgra = {
            rgba.chunks_exact_mut(4).for_each(|pixel| {
                pixel.swap(0, 2);
                mask.push(u8::MAX - pixel[3]);
            });
            rgba
        };
        log::trace!("Creating new native icon");
        //  LoadIconW(None, IDI_QUESTION)?
        let handle = error_check(unsafe {
            CreateIcon(
                null_mut(),
                width as i32,
                height as i32,
                1,
                4 * u8::BITS as u8,
                mask.as_ptr(),
                bgra.as_ptr()
            )
        })?;
        Ok(Self {
            handle: Arc::new(NativeIconHandle(handle))
        })
    }

    pub fn from_resource(resource_id: u16, size: Option<(u32, u32)>) -> TrayResult<Self> {
        let (width, height) = size.unwrap_or((0, 0));
        log::trace!("Creating new native icon");
        //  LoadIconW(None, IDI_QUESTION)?
        let handle = error_check(unsafe {
            LoadImageW(
                get_instance_handle(),
                resource_id as *const u16,
                IMAGE_ICON,
                width as i32,
                height as i32,
                LR_DEFAULTSIZE
            )
        })?;
        Ok(Self {
            handle: Arc::new(NativeIconHandle(handle))
        })
    }

    pub fn handle(&self) -> HICON {
        self.handle.0
    }
}

#[derive(Eq, PartialEq)]
struct NativeIconHandle(HICON);

unsafe impl Send for NativeIconHandle {}
unsafe impl Sync for NativeIconHandle {}

impl Drop for NativeIconHandle {
    fn drop(&mut self) {
        log::trace!("Dropping native icon");
        if let Err(err) = error_check(unsafe { DestroyIcon(self.0) }) {
            log::warn!("Failed to destroy native icon: {err}");
        }
    }
}

impl Debug for NativeIconHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
