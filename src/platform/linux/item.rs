use crate::platform::linux::{TrayCallback, MENU_PATH};
use crate::{ClickType, TrayEvent};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use zbus::{dbus_interface, SignalContext};

pub struct StatusNotifierItem<T> {
    first_activate: AtomicBool,
    tooltip: Mutex<String>,
    icon: Mutex<String>,
    callback: TrayCallback<T>,
}

impl<T> StatusNotifierItem<T> {
    pub fn new(icon: String, tooltip: String, callback: TrayCallback<T>) -> Self {
        Self {
            first_activate: AtomicBool::new(true),
            tooltip: Mutex::new(tooltip),
            icon: Mutex::new(icon),
            callback,
        }
    }
}

impl<T: Send + 'static> StatusNotifierItem<T> {
    pub async fn update_tooltip(
        &self,
        tooltip: String,
        signal_context: &SignalContext<'_>,
    ) -> zbus::Result<()> {
        *self.tooltip.lock() = tooltip;
        Self::new_tool_tip(signal_context).await?;
        Ok(())
    }

    pub async fn update_icon(
        &self,
        icon: String,
        signal_context: &SignalContext<'_>,
    ) -> zbus::Result<()> {
        *self.icon.lock() = icon;
        Self::new_icon(signal_context).await?;
        Ok(())
    }
}

#[dbus_interface(name = "org.kde.StatusNotifierItem")]
impl<T: Send + 'static> StatusNotifierItem<T> {
    fn activate(&self, _x: i32, _y: i32) {
        //skipping the first activation, which triggers the construction of the menu
        //after that every activation appears to be a double click
        let first = self.first_activate.swap(false, Ordering::SeqCst);
        if !first {
            (self.callback.lock())(TrayEvent::Tray(ClickType::Double))
        }
    }

    fn context_menu(&self, _x: i32, _y: i32) {
        //println!("context menu {x} {y}");
    }

    fn scroll(&self, _delta: i32, _orientation: &str) {
        //println!("scroll {delta} {orientation}");
    }

    fn secondary_activate(&self, _x: i32, _y: i32) {
        //println!("secondary activate {x} {y}");
    }

    #[dbus_interface(signal)]
    async fn new_attention_icon(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_icon(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_overlay_icon(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_status(ctx: &SignalContext<'_>, status: &str) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_title(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_tool_tip(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(property)]
    fn attention_icon_name(&self) -> String {
        Default::default()
    }

    #[dbus_interface(property)]
    fn attention_icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> {
        Default::default()
    }

    #[dbus_interface(property)]
    fn attention_movie_name(&self) -> String {
        Default::default()
    }

    #[dbus_interface(property)]
    fn category(&self) -> String {
        String::from("ApplicationStatus")
    }

    #[dbus_interface(property)]
    fn icon_name(&self) -> String {
        self.icon.lock().clone()
    }

    #[dbus_interface(property)]
    fn icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> {
        Default::default()
    }

    #[dbus_interface(property)]
    fn icon_theme_path(&self) -> String {
        Default::default()
    }

    #[dbus_interface(property)]
    fn id(&self) -> String {
        //TODO How to handle this id?
        String::from("betrayer")
    }

    #[dbus_interface(property)]
    fn item_is_menu(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn menu(&self) -> OwnedObjectPath {
        ObjectPath::from_str_unchecked(MENU_PATH).into()
    }

    #[dbus_interface(property)]
    fn overlay_icon_name(&self) -> String {
        Default::default()
    }

    #[dbus_interface(property)]
    fn overlay_icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> {
        Default::default()
    }

    #[dbus_interface(property)]
    fn status(&self) -> String {
        String::from("Active")
    }

    #[dbus_interface(property)]
    fn title(&self) -> String {
        String::from("CHECKED!")
    }

    #[dbus_interface(property)]
    fn tool_tip(&self) -> (String, Vec<(i32, i32, Vec<u8>)>, String, String) {
        (
            String::new(),
            Vec::new(),
            self.tooltip.lock().clone(),
            String::new(),
        )
    }

    #[dbus_interface(property)]
    fn window_id(&self) -> i32 {
        0
    }
}
