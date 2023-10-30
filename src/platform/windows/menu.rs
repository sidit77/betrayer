use std::any::Any;
use windows::core::{PCWSTR, Result};
use windows::Win32::UI::WindowsAndMessaging::{AppendMenuW, CreatePopupMenu, DestroyMenu, HMENU, MF_POPUP, MF_SEPARATOR, MF_STRING};
use crate::{Menu, MenuItem};
use crate::platform::windows::encode_wide;

pub struct NativeMenu {
    hmenu: HMENU,
    signals_map: Box<dyn SignalMap>
}

impl NativeMenu {
/*
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

 */
    pub fn hmenu(&self) -> HMENU {
        self.hmenu
    }

    pub fn map(&self, id: u16) -> Option<&dyn Any> {
        self.signals_map.map(id)
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

fn add_all<T>(hmenu: HMENU, signals: &mut Vec<T>, items: Vec<MenuItem<T>>) -> Result<()> {
    for item in items {
        match item {
            MenuItem::Separator => {
                unsafe { AppendMenuW(hmenu, MF_SEPARATOR, 0, None)? };
            }
            MenuItem::Button { name, signal } => {
                let wide = encode_wide(&name);
                unsafe { AppendMenuW(hmenu, MF_STRING, signals.len(), PCWSTR(wide.as_ptr()))? };
                signals.push(signal);
            }
            MenuItem::Menu { name, children } => {
                let submenu = unsafe { CreatePopupMenu()? };
                add_all(submenu, signals, children)?;
                let wide = encode_wide(&name);
                unsafe { AppendMenuW(hmenu, MF_POPUP, submenu.0 as _, PCWSTR(wide.as_ptr()))? };
            }
        }
    }
    Ok(())
}

impl<T: 'static> TryFrom<Menu<T>> for NativeMenu {
    type Error = windows::core::Error;

    fn try_from(value: Menu<T>) -> std::result::Result<Self, Self::Error> {
        let hmenu = unsafe { CreatePopupMenu()? };
        let mut signals = Vec::<T>::new();
        add_all(hmenu, &mut signals, value.items)?;
        Ok(Self {
            hmenu,
            signals_map: Box::new(signals),
        })
    }
}

trait SignalMap {
    fn map(&self, id: u16) -> Option<&dyn Any>;
}

impl<T: 'static> SignalMap for Vec<T> {
    fn map(&self, id: u16) -> Option<&dyn Any> {
        self
            .get(id as usize)
            .map(|r| r as _)
    }
}