#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{NativeIcon, NativeTrayIcon, PlatformError};
#[cfg(target_os = "macos")]
pub use macos::{NativeIcon, NativeTrayIcon, PlatformError};
#[cfg(target_os = "windows")]
pub use windows::{NativeIcon, NativeTrayIcon, PlatformError};
