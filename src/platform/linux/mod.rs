mod menu;

use std::sync::atomic::{AtomicUsize, Ordering};
use zbus::{Connection, ConnectionBuilder, dbus_interface, dbus_proxy, SignalContext};
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use crate::error::{ErrorSource, TrayResult};
use crate::{Menu, TrayEvent, TrayIconBuilder};
use crate::platform::linux::menu::DBusMenu;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

pub struct NativeTrayIcon<T> {
    signal: Vec<T>,
    connection: Connection
}

impl<T: Clone + Send + 'static> NativeTrayIcon<T> {

    pub async fn new_async<F>(builder: TrayIconBuilder<T>, callback: F) -> TrayResult<Self>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        let name = format!(
            "org.kde.StatusNotifierItem-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::AcqRel)
        );

        let conn = ConnectionBuilder::session()?
            .name(name.clone())?
            .serve_at("/StatusNotifierItem", StatusNotifierItem)?
            .serve_at("/MenuBar", DBusMenu::new(builder.menu
                .unwrap_or_else(Menu::empty), callback))?
            //.internal_executor(false)
            .build()
            .await?;

        let proxy = StatusNotifierWatcherProxy::builder(&conn)
            .path("/StatusNotifierWatcher")?
            .build()
            .await?;

        println!("{:?}", proxy.register_status_notifier_item(&name).await);



        Ok(Self {
            signal: vec![],
            connection: conn,
        })

    }

    pub fn new<F>(builder: TrayIconBuilder<T>, callback: F) -> TrayResult<Self>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        async_io::block_on(Self::new_async(builder, callback))
    }

}

impl<T> NativeTrayIcon<T> {
    pub fn set_tooltip(&self, _tooltip: Option<String>) {

    }

}

impl<T: 'static> NativeTrayIcon<T> {
    pub fn set_menu(&self, _menu: Option<Menu<T>>) {

    }
}

struct StatusNotifierItem;

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
        ObjectPath::from_str_unchecked("/MenuBar").into()
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


#[dbus_proxy(interface = "org.kde.StatusNotifierWatcher", assume_defaults = true)]
trait StatusNotifierWatcher {

    fn register_status_notifier_host(&self, service: &str) -> zbus::Result<()>;

    fn register_status_notifier_item(&self, service: &str) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn status_notifier_host_registered(&self) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn status_notifier_host_unregistered(&self) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn status_notifier_item_registered(&self, arg_1: &str) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn status_notifier_item_unregistered(&self, arg_1: &str) -> zbus::Result<()>;

    #[dbus_proxy(property)]
    fn is_status_notifier_host_registered(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property)]
    fn protocol_version(&self) -> zbus::Result<i32>;

    #[dbus_proxy(property)]
    fn registered_status_notifier_items(&self) -> zbus::Result<Vec<String>>;
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NativeIcon;

impl NativeIcon {
    pub fn from_rgba(_rgba: Vec<u8>, _width: u32, _height: u32) -> TrayResult<Self> {
        Ok(Self)
    }
}


pub type PlatformError = zbus::Error;
impl From<PlatformError> for ErrorSource {
    fn from(value: PlatformError) -> Self {
        ErrorSource::Os(value)
    }
}