use std::any::Any;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, SetForegroundWindow, TrackPopupMenu, HMENU, MF_CHECKED, MF_POPUP, MF_SEPARATOR,
    MF_STRING, TPM_BOTTOMALIGN, TPM_LEFTALIGN
};

use crate::error::{TrayError, TrayResult};
use crate::platform::windows::encode_wide;
use crate::{Menu, MenuItem};

pub struct NativeMenu {
    hmenu: HMENU,
    signals_map: Box<dyn SignalMap>
}

impl NativeMenu {
    pub fn show_on_cursor(&self, hwnd: HWND) -> TrayResult<()> {
        let mut cursor = POINT::default();
        unsafe {
            GetCursorPos(&mut cursor)?;
            SetForegroundWindow(hwnd).ok()?;
            TrackPopupMenu(self.hmenu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, cursor.x, cursor.y, 0, hwnd, None).ok()?;
        }
        Ok(())
    }

    pub fn map(&self, id: u16) -> Option<&dyn Any> {
        self.signals_map.map(id)
    }
}

impl Drop for NativeMenu {
    fn drop(&mut self) {
        log::trace!("Destroying native menu");
        unsafe {
            DestroyMenu(self.hmenu).unwrap_or_else(|err| log::warn!("Failed to destroy native menu: {err}"));
        }
    }
}

fn add_all<T>(hmenu: HMENU, signals: &mut Vec<T>, items: Vec<MenuItem<T>>) -> TrayResult<()> {
    for item in items {
        match item {
            MenuItem::Separator => {
                unsafe { AppendMenuW(hmenu, MF_SEPARATOR, 0, None)? };
            }
            MenuItem::Button { name, signal, checked } => {
                let checked = checked
                    .map(|v| v.then_some(MF_CHECKED).unwrap_or_default())
                    .unwrap_or_default();
                let wide = encode_wide(&name);
                unsafe { AppendMenuW(hmenu, MF_STRING | checked, signals.len(), PCWSTR(wide.as_ptr()))? };
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
    type Error = TrayError;

    fn try_from(value: Menu<T>) -> Result<Self, Self::Error> {
        log::trace!("Creating new native menu");
        let hmenu = unsafe { CreatePopupMenu()? };
        let mut signals = Vec::<T>::new();
        add_all(hmenu, &mut signals, value.items)?;
        Ok(Self {
            hmenu,
            signals_map: Box::new(signals)
        })
    }
}

trait SignalMap {
    fn map(&self, id: u16) -> Option<&dyn Any>;
}

impl<T: 'static> SignalMap for Vec<T> {
    fn map(&self, id: u16) -> Option<&dyn Any> {
        self.get(id as usize).map(|r| r as _)
    }
}
