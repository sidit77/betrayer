use std::mem::size_of;
use once_cell::sync::Lazy;
use windows::core::{PCWSTR, w, Result, s};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;
use windows::Win32::UI::Shell::{DefSubclassProc, NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, SetWindowSubclass, Shell_NotifyIconW};
use windows::Win32::UI::WindowsAndMessaging::{AppendMenuW, CreateMenu, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, HMENU, HWND_MESSAGE, IDI_QUESTION, LoadIconW, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, PostMessageW, RegisterClassW, RegisterWindowMessageA, SetForegroundWindow, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu, TranslateMessage, UnregisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_QUIT, WM_RBUTTONUP, WNDCLASSW};

const TRAY_SUBCLASS_ID: usize = 6001;
const WM_USER_TRAY_ICON: u32 = 6002;

struct TrayData {
    menu: Menu
}

struct Menu {
    hmenu: HMENU
}

impl Menu {
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
}

impl Drop for Menu {
    fn drop(&mut self) {
        println!("dropping menu");
        unsafe {
            DestroyMenu(self.hmenu).unwrap();
        }
    }
}

fn main() -> Result<()> {
    println!("Hello WOrld");

    let hinstance = get_instance_handle();

    unsafe extern "system" fn tray_icon_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    let class_name = w!("tray_icon_window");
    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(tray_icon_window_proc),
        hInstance: hinstance,
        lpszClassName: class_name,
        ..Default::default()
    };

    unsafe { RegisterClassW(&wnd_class); }

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            PCWSTR::null(),
            WINDOW_STYLE::default(),
            0, 0,
            0, 0,
            HWND_MESSAGE,
            HMENU::default(),
            hinstance,
            None
        )
    };
    assert_ne!(hwnd, HWND::default());


    let icon = unsafe { LoadIconW(None, IDI_QUESTION)? };

    let tray_id = 1;
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

    unsafe { Shell_NotifyIconW(NIM_ADD, &notify_icon_data).ok()? };

    let data = TrayData {
        menu: Menu::new()?,
    };

    unsafe {
        SetWindowSubclass(
            hwnd,
            Some(tray_subclass_proc),
            TRAY_SUBCLASS_ID,
            Box::into_raw(Box::new(data)) as _)
            .ok()?;
    }

    unsafe {
        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    let notify_icon_data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: tray_id,
        ..Default::default()
    };

    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &notify_icon_data).ok()?;
        DestroyWindow(hwnd)?;
        UnregisterClassW(class_name, hinstance)?;
    };


    Ok(())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ClickType {
    Left,
    Right,
    Double
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
    //println!("msg: {}", msg);
    match msg {
        WM_DESTROY => {
            drop(Box::from_raw(subclass_input_ptr));
            println!("Destroyed");
        },
        _ if msg == *S_U_TASKBAR_RESTART => println!("Taskbar restarted"),
        WM_USER_TRAY_ICON => if let Some(click) = ClickType::from_lparam(lparam) {
            println!("click: {:?}", click);
            if click == ClickType::Right {
                let mut cursor = POINT::default();
                GetCursorPos(&mut cursor).unwrap();
                SetForegroundWindow(hwnd).unwrap();
                let hmenu = (*subclass_input_ptr).menu.hmenu;
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
        WM_COMMAND => {
            let id = LOWORD(wparam.0 as _);
            println!("Command: {}", id);
            if id == 42 {
                PostMessageW(HWND::default(), WM_QUIT, WPARAM::default(), LPARAM::default()).unwrap();
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

static S_U_TASKBAR_RESTART: Lazy<u32> = Lazy::new(|| unsafe { RegisterWindowMessageA(s!("TaskbarCreated")) });

// taken from winit's code base
// https://github.com/rust-windowing/winit/blob/ee88e38f13fbc86a7aafae1d17ad3cd4a1e761df/src/platform_impl/windows/util.rs#L138
pub fn get_instance_handle() -> HINSTANCE {
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