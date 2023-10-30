use windows::core::{Result, w};
use windows::Win32::UI::WindowsAndMessaging::{AppendMenuW, CreatePopupMenu, DestroyMenu, HMENU, MF_POPUP, MF_SEPARATOR, MF_STRING};

pub struct NativeMenu {
    hmenu: HMENU
}

impl NativeMenu {
    pub fn new() -> Result<Self> {
        unsafe {
            let hmenu = CreatePopupMenu()?;

            let submenu = CreatePopupMenu()?;

            AppendMenuW(submenu, MF_STRING, 39, w!("Profile 1")).unwrap();

            AppendMenuW(hmenu, MF_POPUP, submenu.0 as _, w!("Profiles")).unwrap();
            AppendMenuW(hmenu, MF_SEPARATOR, 0, None).unwrap();
            AppendMenuW(hmenu, MF_STRING, 41, w!("Open")).unwrap();
            AppendMenuW(hmenu, MF_STRING, 42, w!("Quit")).unwrap();

            Ok(Self {
                hmenu
            })
        }
    }

    pub fn hmenu(&self) -> HMENU {
        self.hmenu
    }
}

impl Drop for NativeMenu {
    fn drop(&mut self) {
        println!("dropping menu");
        unsafe {
            DestroyMenu(self.hmenu).unwrap();
        }
    }
}