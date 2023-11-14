use zbus::{dbus_interface, SignalContext};
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use crate::platform::linux::MENU_PATH;

pub struct StatusNotifierItem {
    tooltip: String
}

impl StatusNotifierItem {
    pub fn new(tooltip: String) -> Self {
        Self {
            tooltip,
        }
    }
}

#[dbus_interface(name = "org.kde.StatusNotifierItem")]
impl StatusNotifierItem {

    fn activate(&self, x: i32, y: i32) {
        println!("activate {x} {y}");
    }

    fn context_menu(&self, x: i32, y: i32) {
        println!("context menu {x} {y}");
    }

    fn scroll(&self, delta: i32, orientation: &str) {
        println!("scroll {delta} {orientation}");
    }

    fn secondary_activate(&self, x: i32, y: i32) {
        println!("secondary activate {x} {y}");
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
        String::from("help-about")
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
        Default::default()
    }

    #[dbus_interface(property)]
    fn window_id(&self) -> i32 {
        0
    }
}

