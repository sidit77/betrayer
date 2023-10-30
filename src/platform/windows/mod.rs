mod menu;

use std::any::Any;
use std::iter::once;
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Once;
use once_cell::sync::Lazy;
use windows::core::{PCWSTR, w};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;
use windows::Win32::UI::Shell::{DefSubclassProc, NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, SetWindowSubclass, Shell_NotifyIconW};
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, HMENU, HWND_MESSAGE, IDI_QUESTION, LoadIconW, RegisterClassW, RegisterWindowMessageW, SetForegroundWindow, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW};
use crate::platform::windows::menu::NativeMenu;
use crate::{ClickType, TrayEvent, TrayIconBuilder};

const TRAY_SUBCLASS_ID: usize = 6001;
const WM_USER_TRAY_ICON: u32 = 6002;

pub struct NativeTrayIcon {
    hwnd: HWND,
    tray_id: u32
}

impl NativeTrayIcon {

    pub fn new<T, F>(builder: TrayIconBuilder<T>, mut callback: F) -> Self
        where F: FnMut(TrayEvent<T>) + Send + 'static,
              T: Clone + 'static
    {

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
        assert_ne!(hwnd, HWND::default());

        let icon = unsafe { LoadIconW(None, IDI_QUESTION).unwrap() };

        let tray_id = GLOBAL_TRAY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let notify_icon_data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            uFlags: NIF_MESSAGE | NIF_ICON/* | NIF_TIP*/,
            hWnd: hwnd,
            uID: tray_id,
            hIcon: icon,
            /*szTip: */
            uCallbackMessage: WM_USER_TRAY_ICON,
            ..Default::default()
        };

        unsafe { Shell_NotifyIconW(NIM_ADD, &notify_icon_data).ok().unwrap() };

        let menu = builder
            .menu
            .map(NativeMenu::try_from)
            .transpose()
            .unwrap();

        let erased_callback: Box<dyn FnMut(TrayEvent<&dyn Any>) + 'static> = Box::new(move |event: TrayEvent<&dyn Any> | {
            let event = match event {
                TrayEvent::Menu(signal) => TrayEvent::Menu(signal
                    .downcast_ref::<T>()
                    .expect("")
                    .clone()),
                TrayEvent::Tray(click) => TrayEvent::Tray(click)
            };
            callback(event);
        });

        let data = TrayData {
            menu,
            callback: erased_callback,
        };

        unsafe {
            SetWindowSubclass(
                hwnd,
                Some(tray_subclass_proc),
                TRAY_SUBCLASS_ID,
                Box::into_raw(Box::new(data)) as _)
                .ok().unwrap();
        }

        NativeTrayIcon {
            hwnd,
            tray_id,
        }

    }

}

impl Drop for NativeTrayIcon {
    fn drop(&mut self) {
        let notify_icon_data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: self.tray_id,
            ..Default::default()
        };

        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &notify_icon_data).ok().unwrap();
            DestroyWindow(self.hwnd).unwrap();
        };
    }
}

struct TrayData {
    menu: Option<NativeMenu>,
    callback: Box<dyn FnMut(TrayEvent<&dyn Any>) + 'static>
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

unsafe extern "system" fn tray_subclass_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _id: usize, subclass_input_ptr: usize) -> LRESULT {
    let subclass_input_ptr = subclass_input_ptr as *mut TrayData;
    let subclass_input = &mut *subclass_input_ptr;
    //println!("msg: {}", msg);
    match msg {
        WM_DESTROY => {
            drop(Box::from_raw(subclass_input_ptr));
            println!("Destroyed");
        },
        _ if msg == *S_U_TASKBAR_RESTART => println!("Taskbar restarted"),
        WM_USER_TRAY_ICON => if let Some(click) = ClickType::from_lparam(lparam) {
            (subclass_input.callback)(TrayEvent::Tray(click));
            if click == ClickType::Right {
                if let Some(menu) =  subclass_input.menu.as_ref() {
                    let mut cursor = POINT::default();
                    GetCursorPos(&mut cursor).unwrap();
                    SetForegroundWindow(hwnd).unwrap();
                    let hmenu = menu.hmenu();
                    TrackPopupMenu(
                        hmenu,
                        TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                        cursor.x, cursor.y,
                        0,
                        hwnd,
                        None
                    ).unwrap();
                }
            }
        }
        WM_COMMAND => {
            let id = LOWORD(wparam.0 as _);
            println!("Command: {}", id);
            //if id == 42 {
            //    PostMessageW(HWND::default(), WM_QUIT, WPARAM::default(), LPARAM::default()).unwrap();
            //}
            if let Some(menu) = subclass_input.menu.as_ref() {
                match menu.map(id) {
                    None => println!("Unknown id"),
                    Some(signal) => (subclass_input.callback)(TrayEvent::Menu(signal))
                }
            }
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
        unsafe { RegisterClassW(&wnd_class); }
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