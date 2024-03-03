use crate::error::TrayResult;
use std::mem::size_of;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW, NOTIFY_ICON_MESSAGE,
};
use windows::Win32::UI::WindowsAndMessaging::HICON;

pub enum DataAction {
    Add,
    Modify,
    Remove,
}

impl From<DataAction> for NOTIFY_ICON_MESSAGE {
    fn from(value: DataAction) -> Self {
        match value {
            DataAction::Add => NIM_ADD,
            DataAction::Modify => NIM_MODIFY,
            DataAction::Remove => NIM_DELETE,
        }
    }
}

pub struct TrayIconData(NOTIFYICONDATAW);

impl Default for TrayIconData {
    fn default() -> Self {
        Self(NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            ..Default::default()
        })
    }
}

impl TrayIconData {
    pub fn with_message(mut self, message: u32) -> Self {
        self.0.uFlags |= NIF_MESSAGE;
        self.0.uCallbackMessage = message;
        self
    }

    pub fn with_icon(mut self, icon: HICON) -> Self {
        self.0.uFlags |= NIF_ICON;
        self.0.hIcon = icon;
        self
    }

    pub fn with_tooltip<S: AsRef<str>>(mut self, tooltip: S) -> Self {
        self.0.uFlags |= NIF_TIP;
        tooltip
            .as_ref()
            .encode_utf16()
            .take(self.0.szTip.len() - 1)
            .enumerate()
            .for_each(|(i, c)| self.0.szTip[i] = c);
        self
    }

    pub fn apply(mut self, hwnd: HWND, id: u32, action: DataAction) -> TrayResult<()> {
        self.0.hWnd = hwnd;
        self.0.uID = id;
        unsafe { Shell_NotifyIconW(action.into(), &self.0).ok()? };
        Ok(())
    }
}
