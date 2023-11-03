use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use zbus::{Connection, ConnectionBuilder, dbus_interface, dbus_proxy, SignalContext};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Str, Value};
use crate::error::{ErrorSource, TrayResult};
use crate::{Menu, TrayEvent, TrayIconBuilder};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

pub struct NativeTrayIcon<T> {
    signal: Vec<T>,
    connection: Connection
}

impl<T: Clone + 'static> NativeTrayIcon<T> {

    pub async fn new_async<F>(builder: TrayIconBuilder<T>, mut callback: F) -> TrayResult<Self>
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
            .serve_at("/MenuBar", DBusMenu)?
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

struct DBusMenu;

#[dbus_interface(name = "com.canonical.dbusmenu")]
impl DBusMenu {

    fn get_layout(&self, parent_id: i32, recursion_depth: i32, property_names: Vec<&str>) -> (u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>)) {
        println!("get layout {:?} {:?} {:?}", parent_id, recursion_depth, property_names);
        (0, (0, HashMap::from([
            //(String::from("type"), OwnedValue::from(Str::from_static("standard"))),
            //(String::from("label"), OwnedValue::from(Str::from_static("Hello World")))
            (String::from("children-display"), OwnedValue::from(Str::from_static("submenu")))
        ]), vec![Value::new((1, HashMap::<String, OwnedValue>::new(), Vec::<OwnedValue>::new())).to_owned()]))
    }

    fn get_group_properties(&self, ids: Vec<i32>, property_names: Vec<&str>) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        println!("get group properties {:?} {:?}", ids, property_names);
        vec![
            (1, HashMap::from([
                (String::from("label"), OwnedValue::from(Str::from_static("Hello World")))
            ]))
        ]
    }

    fn get_property(&self, id: i32, name: &str) -> OwnedValue {
        println!("get property {} {}", id, name);
        OwnedValue::from(0)
    }

    fn event(&self, id: i32, event_id: &str, data: Value<'_>, timestamp: u32) {
        println!("event: {} {}", id, event_id);
    }

    fn event_group(&self, events: Vec<(i32, &str, Value<'_>, u32)>) -> Vec<i32> {
        for (id, event, data, timestamp) in events {
            self.event(id, event, data, timestamp);
        }
        Vec::new()
    }

    fn about_to_show(&self, id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        Default::default()
    }


    #[dbus_interface(signal)]
    async fn item_activation_requested(ctx: &SignalContext<'_>, id: i32, timestamp: u32) -> zbus::Result<()> { }

    #[dbus_interface(signal)]
    async fn items_properties_updated(ctx: &SignalContext<'_>, updated_props: &[(i32, HashMap<&str, Value<'_>>, )], removed_props: &[(i32, &[&str])]) -> zbus::Result<()> { }

    #[dbus_interface(signal)]
    async fn layout_updated(ctx: &SignalContext<'_>, revision: u32, parent: i32) -> zbus::Result<()> { }

    #[dbus_interface(property)]
    fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }

    #[dbus_interface(property)]
    fn status(&self) -> String {
        String::from("normal")
    }

    #[dbus_interface(property)]
    fn text_direction(&self) -> String {
        String::from("ltr")
    }

    #[dbus_interface(property)]
    fn version(&self) -> u32 {
        3
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