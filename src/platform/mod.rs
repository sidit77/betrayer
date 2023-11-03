#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
pub use windows::{NativeTrayIcon, NativeIcon, PlatformError};

#[cfg(target_os = "linux")]
pub use linux::{NativeTrayIcon, NativeIcon, PlatformError};