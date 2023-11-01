mod menu;
mod tray;
mod icon;

use std::any::Any;
use std::cell::Cell;
use std::iter::once;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Once;
use once_cell::sync::Lazy;
use windows::core::{PCWSTR, w};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, DestroyWindow, HMENU, HWND_MESSAGE, RegisterClassW, RegisterWindowMessageW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW};
use crate::platform::windows::menu::NativeMenu;
use crate::{ClickType, ensure, Menu, TrayEvent, TrayIconBuilder};
use crate::error::{ErrorSource, TrayError, TrayResult};
use crate::platform::windows::tray::{DataAction, TrayIconData};
use crate::utils::OptionCellExt;

pub use icon::NativeIcon;

//TODO Better error handling for the set_* functions
//TODO Replace Cell to avoid potential overrides

const TRAY_SUBCLASS_ID: usize = 6001;
const WM_USER_TRAY_ICON: u32 = 6002;

pub struct NativeTrayIcon<T> {
    hwnd: HWND,
    tray_id: u32,
    shared: Rc<SharedTrayData>,
    _signal_type: PhantomData<T>
}

struct TrayLoopData {
    shared: Rc<SharedTrayData>,
    callback: Box<dyn FnMut(TrayEvent<&dyn Any>) + 'static>
}

struct SharedTrayData {
    menu: Cell<Option<NativeMenu>>,
    tooltip: Cell<Option<String>>,
    icon: Cell<Option<NativeIcon>>
}

impl<T: Clone + 'static> NativeTrayIcon<T> {

    pub fn new<F>(builder: TrayIconBuilder<T>, mut callback: F) -> TrayResult<Self>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        let tray_id = GLOBAL_TRAY_COUNTER.fetch_add(1, Ordering::Relaxed);

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                get_class_name(),
                PCWSTR::null(),
                WINDOW_STYLE::default(),
                0, 0,
                0, 0,
                HWND_MESSAGE,
                HMENU::default(),
                get_instance_handle(),
                None
            )
        };
        ensure!(hwnd != HWND::default(), TrayError::custom("Invalid HWND"));
        log::trace!("Created new message window (tray id: {tray_id})");

        let shared = Rc::new(SharedTrayData {
            menu: Cell::new(builder
                .menu
                .map(NativeMenu::try_from)
                .transpose()?),
            tooltip: Cell::new(builder.tooltip),
            icon: Cell::new(builder.icon.map(NativeIcon::from)),
        });

        TrayIconData::from(&shared)
            .with_message(WM_USER_TRAY_ICON)
            .apply(hwnd, tray_id, DataAction::Add)?;


        let data = TrayLoopData {
            shared: shared.clone(),
            callback: Box::new(move |event: TrayEvent<&dyn Any> | {
                let event = match event {
                    TrayEvent::Menu(signal) => TrayEvent::Menu(signal
                        .downcast_ref::<T>()
                        .expect("Signal has the wrong type")
                        .clone()),
                    TrayEvent::Tray(click) => TrayEvent::Tray(click)
                };
                callback(event);
            }),
        };

        unsafe {
            SetWindowSubclass(
                hwnd,
                Some(tray_subclass_proc),
                TRAY_SUBCLASS_ID,
                Box::into_raw(Box::new(data)) as _)
                .ok()?;
        }

        Ok(NativeTrayIcon {
            hwnd,
            tray_id,
            shared,
            _signal_type: PhantomData::default(),
        })

    }

}

impl<T> NativeTrayIcon<T> {
    pub fn set_tooltip(&self, tooltip: Option<String>) {
        TrayIconData::default()
            .with_tooltip(tooltip
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(""))
            .apply(self.hwnd, self.tray_id, DataAction::Modify)
            .unwrap();
        self.shared.tooltip.set(tooltip)
    }

}

impl<T: 'static> NativeTrayIcon<T> {
    pub fn set_menu(&self, menu: Option<Menu<T>>) {
        let menu = menu
            .map(|m| NativeMenu::try_from(m).unwrap());
        self.shared.menu.set(menu);
    }
}

impl<T> Drop for NativeTrayIcon<T> {
    fn drop(&mut self) {
        log::trace!("Destroying message window (tray id: {})", self.tray_id);

        TrayIconData::default()
            .apply(self.hwnd, self.tray_id, DataAction::Remove)
            .unwrap_or_else(|err| log::warn!("Failed to remove tray icon: {err}"));

        unsafe {
            DestroyWindow(self.hwnd)
                .unwrap_or_else(|err| log::warn!("Failed to destroy message window: {err}"));
        };
    }
}





unsafe extern "system" fn tray_subclass_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _id: usize, subclass_input_ptr: usize) -> LRESULT {
    let subclass_input_ptr = subclass_input_ptr as *mut TrayLoopData;
    let subclass_input = &mut *subclass_input_ptr;
    match msg {
        WM_DESTROY => {
            drop(Box::from_raw(subclass_input_ptr));
            log::trace!("Dropped message loop data");
        },
        _ if msg == *S_U_TASKBAR_RESTART => log::debug!("Taskbar restarted"),
        WM_USER_TRAY_ICON => if let Some(click) = ClickType::from_lparam(lparam) {
            (subclass_input.callback)(TrayEvent::Tray(click));
            if click == ClickType::Right {
                subclass_input
                    .shared
                    .menu
                    .with(|menu| menu
                        .show_on_cursor(hwnd)
                        .unwrap_or_else(|err| log::warn!("Failed to show menu: {err}")));
            }
        }
        WM_COMMAND => {
            let id = LOWORD(wparam.0 as _);
            subclass_input
                .shared
                .menu
                .with(|menu| {
                    match menu.map(id) {
                        None => log::debug!("Unknown menu item id: {id}"),
                        Some(signal) => (subclass_input.callback)(TrayEvent::Menu(signal))
                    }
                });
        }
        _ => {}
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}


#[allow(non_snake_case)]
pub fn LOWORD(dword: u32) -> u16 {
    (dword & 0xFFFF) as u16
}

static GLOBAL_TRAY_COUNTER: AtomicU32 = AtomicU32::new(1);

static S_U_TASKBAR_RESTART: Lazy<u32> = Lazy::new(|| unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) });


fn get_class_name() -> PCWSTR {
    static INITIALIZED: Once = Once::new();

    let class_name = w!("tray_icon_window");

    INITIALIZED.call_once(|| {
        let hinstance = get_instance_handle();

        unsafe extern "system" fn tray_icon_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(tray_icon_window_proc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };
        let class = unsafe { RegisterClassW(&wnd_class) };
        log::trace!("Registered tray window class: 0x{:x}", class);
    });

    class_name
}

fn encode_wide(string: &str) -> Vec<u16> {
    string
        .encode_utf16()
        .chain(once(0))
        .collect()
}

// taken from winit's code base
// https://github.com/rust-windowing/winit/blob/ee88e38f13fbc86a7aafae1d17ad3cd4a1e761df/src/platform_impl/windows/util.rs#L138
fn get_instance_handle() -> HINSTANCE {
    // Gets the instance handle by taking the address of the
    // pseudo-variable created by the microsoft linker:
    // https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483

    // This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
    // https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance

    extern "C" {
        static __ImageBase: IMAGE_DOS_HEADER;
    }

    HINSTANCE(unsafe { &__ImageBase as *const _ as _ })
}


pub type PlatformError = windows::core::Error;
impl From<PlatformError> for ErrorSource {
    fn from(value: PlatformError) -> Self {
        ErrorSource::Os(value)
    }
}

impl ClickType {
    fn from_lparam(lparam: LPARAM) -> Option<Self> {
        match lparam.0 as u32 {
            WM_LBUTTONUP => Some(Self::Left),
            WM_RBUTTONUP => Some(Self::Right),
            WM_LBUTTONDBLCLK => Some(Self::Double),
            _ => None
        }
    }
}

impl<T: AsRef<SharedTrayData>> From<T> for TrayIconData {
    fn from(value: T) -> Self {
        let shared = value.as_ref();
        let mut data = Some(TrayIconData::default());
        shared.tooltip.with(|tooltip| {
            let t = data
                .take()
                .unwrap()
                .with_tooltip(tooltip);
            data = Some(t);
        });
        shared.icon.with(|icon| {
            let t = data
                .take()
                .unwrap()
                .with_icon(icon.handle());
            data = Some(t);
        });
        data.unwrap()
    }
}