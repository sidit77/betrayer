use crate::error::{TrayError, TrayResult};
use crate::platform::{NativeIcon, NativeTrayIcon};

mod platform;
mod error;
mod utils;

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

    pub fn with_menu(mut self, menu: Menu<T>) -> Self {
        self.menu = Some(menu);
        self
    }

    pub fn with_tooltip<S: ToString>(mut self, tooltip: S) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

}

impl<T: Clone + Send + 'static> TrayIconBuilder<T> {

    pub fn build<F>(self, callback: F) -> TrayResult<TrayIcon<T>>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        Ok(TrayIcon(NativeTrayIcon::new(self, callback)?))
    }

}

pub struct TrayIcon<T>(NativeTrayIcon<T>);

impl<T> TrayIcon<T> {
    pub fn set_tooltip<S: ToString>(&self, tooltip: impl Into<Option<S>>) {
        self.0.set_tooltip(tooltip.into().map(|s| s.to_string()))
    }
}

impl<T: 'static> TrayIcon<T> {
    pub fn set_menu(&self, menu: impl Into<Option<Menu<T>>>) {
        self.0.set_menu(menu.into())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClickType {
    Left,
    Right,
    Double
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TrayEvent<T> {
    Tray(ClickType),
    Menu(T)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Menu<T> {
    items: Vec<MenuItem<T>>
}

impl<T> Menu<T> {
    pub fn new<I>(items: I) -> Self
        where I: IntoIterator<Item=MenuItem<T>>
    {
        Self {
            items: items.into_iter().collect(),
        }
    }
    
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
        }
    }    
    
}

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

    pub fn separator() -> Self {
        Self::Separator
    }

    pub fn button<S>(name: S, signal: T) -> Self
        where S: ToString
    {
        Self::Button {
            name: name.to_string(),
            signal,
            checked: false,
        }
    }

    pub fn check_button<S>(name: S, signal: T, checked: bool) -> Self
        where S: ToString
    {
        Self::Button {
            name: name.to_string(),
            signal,
            checked,
        }
    }

    pub fn menu<S, I>(name: S, children: I) -> Self
        where S: ToString, I: IntoIterator<Item=MenuItem<T>>
    {
        Self::Menu {
            name: name.to_string(),
            children: children.into_iter().collect(),
        }
    }

}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Icon(NativeIcon);

impl Icon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> TrayResult<Self> {
        ensure!(rgba.len() as u32 == width * height * 4, TrayError::custom("Invalid dimensions"));
        Ok(Icon(NativeIcon::from_rgba(rgba, width, height)?))
    }

    #[cfg(target_os = "linux")]
    pub fn from_png_bytes(bytes: &[u8]) -> TrayResult<Self> {
        Ok(Icon(NativeIcon::from_png_bytes(bytes)?))
    }

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