#![doc = include_str!("../Readme.md")]

mod platform;
mod error;
mod utils;

#[cfg(feature = "winit")]
pub mod winit;

use platform::{NativeIcon, NativeTrayIcon};

pub use error::{TrayResult, TrayError, ErrorSource};

/// Builder struct for a tray icon
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TrayIconBuilder<T = ()> {
    menu: Option<Menu<T>>,
    tooltip: Option<String>,
    icon: Option<Icon>
}

impl<T> TrayIconBuilder<T> {

    pub fn new() -> Self {
        Self {
            menu: None,
            tooltip: None,
            icon: None,
        }
    }

    /// Sets the [Menu] of the tray icon. When not set the tray is created without menu.
    pub fn with_menu(mut self, menu: Menu<T>) -> Self {
        self.menu = Some(menu);
        self
    }

    /// Sets the tooltip that appears when hovering over the tray icon.
    ///
    /// Does nothing on MacOS and Linux behaviour depends on the desktop environment
    pub fn with_tooltip<S: ToString>(mut self, tooltip: S) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }

    /// Sets the icon of the tray icon
    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

}

impl<T: Clone + Send + 'static> TrayIconBuilder<T> {

    /// Attempts to create the tray icon. See the the *Platform notes* section of the Readme for more information.
    pub fn build<F>(self, callback: F) -> TrayResult<TrayIcon<T>>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        Ok(TrayIcon(NativeTrayIcon::new(self, callback)?))
    }

}

pub struct TrayIcon<T>(NativeTrayIcon<T>);

impl<T> TrayIcon<T> {

    /// Updates or removes the tooltip
    pub fn set_tooltip<S: ToString>(&self, tooltip: impl Into<Option<S>>) {
        self.0.set_tooltip(tooltip.into().map(|s| s.to_string()))
    }
}

impl<T: 'static> TrayIcon<T> {

    /// Updates or removes the menu
    pub fn set_menu(&self, menu: impl Into<Option<Menu<T>>>) {
        self.0.set_menu(menu.into())
    }
}

/// Enum for describing how a user clicked on the tray icon
///
/// **WARNING**: Click handling has major platform differences.
///
/// On *Windows* this works as you'd expect it to work
///
/// On *Linux* [ClickType::Double] should work as expect and [ClickType::Left] gets emitted by every action that opens the root menu. [ClickType::Right] never gets emitted.
///
/// On *Mac* [ClickType::Left] gets emitted by any actions that would open the menu, but **only** if no menu is present.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClickType {
    Left,
    Right,
    Double
}

/// An event describing how the user interacted with the tray icon or associated menu
///
/// See the docs of [ClickType] for platform specific notes about mouse click events
///
/// The parameter of the [TrayEvent::Menu] variant is a copy of the signal token assigned to the respective [MenuItem]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TrayEvent<T> {
    Tray(ClickType),
    Menu(T)
}

/// A struct describing the layout of a tray icon menu
///
/// The actual Menus are created lazily by the [TrayIcon].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Menu<T> {
    items: Vec<MenuItem<T>>
}

impl<T> Menu<T> {

    /// Create a new menu with the given children
    pub fn new<I>(items: I) -> Self
        where I: IntoIterator<Item=MenuItem<T>>
    {
        Self {
            items: items.into_iter().collect(),
        }
    }

    /// Creates a new empty menu
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
        }
    }    
    
}

/// Various menu items that can be added to a [Menu]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MenuItem<T> {
    Separator,
    Button {
        name: String,
        signal: T,
        checked: bool
    },
    Menu {
        name: String,
        children: Vec<MenuItem<T>>
    }
}

impl<T> MenuItem<T> {

    /// A separator
    pub fn separator() -> Self {
        Self::Separator
    }

    /// A new clickable entry with label that emits a [TrayEvent::Menu] when clicked
    pub fn button<S>(name: S, signal: T) -> Self
        where S: ToString
    {
        Self::Button {
            name: name.to_string(),
            signal,
            checked: false,
        }
    }

    /// A new clickable entry with label and checkmark that emits a [TrayEvent::Menu] when clicked
    pub fn check_button<S>(name: S, signal: T, checked: bool) -> Self
        where S: ToString
    {
        Self::Button {
            name: name.to_string(),
            signal,
            checked,
        }
    }

    /// A new submenu
    pub fn menu<S, I>(name: S, children: I) -> Self
        where S: ToString, I: IntoIterator<Item=MenuItem<T>>
    {
        Self::Menu {
            name: name.to_string(),
            children: children.into_iter().collect(),
        }
    }

}

/// An icon struct
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Icon(NativeIcon);

impl Icon {

    /// Creates a new icon from raw RGBA data
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> TrayResult<Self> {
        ensure!(rgba.len() as u32 == width * height * 4, TrayError::custom("Invalid dimensions"));
        Ok(Icon(NativeIcon::from_rgba(rgba, width, height)?))
    }

    /// Creates a new icon from png encoded image data
    #[cfg(target_os = "linux")]
    pub fn from_png_bytes(bytes: &[u8]) -> TrayResult<Self> {
        Ok(Icon(NativeIcon::from_png_bytes(bytes)?))
    }

    /// Creates a new icon from an embedded resource
    #[cfg(target_os = "windows")]
    pub fn from_resource(resource_id: u16, size: Option<(u32, u32)>) -> TrayResult<Self> {
        Ok(Icon(NativeIcon::from_resource(resource_id, size)?))
    }

}

impl From<Icon> for NativeIcon {
    fn from(value: Icon) -> Self {
        value.0
    }
}