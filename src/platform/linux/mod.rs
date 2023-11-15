mod menu;
mod item;

use std::sync::atomic::{AtomicUsize, Ordering};
use flume::Sender;
use zbus::{ConnectionBuilder, dbus_proxy, Task};
use crate::error::{ErrorSource, TrayResult};
use crate::{Menu, TrayEvent, TrayIconBuilder};
use crate::platform::linux::item::StatusNotifierItem;
use crate::platform::linux::menu::DBusMenu;

static MENU_PATH: &'static str = "/MenuBar";
static ITEM_PATH: &'static str = "/StatusNotifierItem";
static COUNTER: AtomicUsize = AtomicUsize::new(1);

enum TrayUpdate<T> {
    Menu(Menu<T>),
    Tooltip(String)
}

pub struct NativeTrayIcon<T> {
    //_signal: Vec<T>,
    //connection: Connection
    _update_task: Task<()>,
    sender: Sender<TrayUpdate<T>>
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
            .serve_at(ITEM_PATH, StatusNotifierItem::new(
                builder.tooltip.unwrap_or_default()))?
            .serve_at(MENU_PATH, DBusMenu::new(builder.menu
                .unwrap_or_else(Menu::empty), callback))?
            .internal_executor(true)
            .build()
            .await?;

        let (sender, receiver) = flume::unbounded();
        let receiver_task = {

            let connection = conn.clone();
            conn.executor().spawn(async move {
                while let Ok(event) = receiver.recv_async().await {
                    match event {
                        TrayUpdate::Menu(menu) => {
                            let iface = connection
                                .object_server()
                                .interface::<_, DBusMenu<T>>(MENU_PATH)
                                .await.unwrap();
                            let iref = iface.get().await;
                            iref.update_menu(menu, iface.signal_context()).await.unwrap();
                        },
                        TrayUpdate::Tooltip(tooltip) => {
                            let iface = connection
                                .object_server()
                                .interface::<_, StatusNotifierItem>(ITEM_PATH)
                                .await.unwrap();
                            let mut iref = iface.get_mut().await;
                            iref.update_tooltip(tooltip, iface.signal_context()).await.unwrap();
                        }
                    }
                }
            }, "event receiver")
        };
        //let _ = sender.send(TrayUpdate::Menu(Menu::empty()));
        //{
        //    let executor = conn.executor().clone();
        //    std::thread::Builder::new()
        //        .name("zbus::Connection executor".into())
        //        .spawn(move || {
        //            println!("Exec start");
        //            async_io::block_on(async move {
        //                // Run as long as there is a task to run.
        //                while !executor.is_empty() {
        //                    executor.tick().await;
        //                }
        //            });
        //            println!("Exec end");
        //        }).unwrap();
        //}

        let proxy = StatusNotifierWatcherProxy::builder(&conn)
            .path("/StatusNotifierWatcher")?
            .build()
            .await?;

        proxy.register_status_notifier_item(&name).await?;


        Ok(Self {
            //_signal: vec![],
            //connection: conn,
            _update_task: receiver_task,
            sender,
        })

    }

    pub fn new<F>(builder: TrayIconBuilder<T>, callback: F) -> TrayResult<Self>
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        async_io::block_on(Self::new_async(builder, callback))
    }

}

impl<T> NativeTrayIcon<T> {
    pub fn set_tooltip(&self, tooltip: Option<String>) {
        self.sender
            .send(TrayUpdate::Tooltip(tooltip.unwrap_or_default()))
            .unwrap_or_else(|err| log::warn!("Failed to send update: {err}"));
    }

    pub fn set_menu(&self, menu: Option<Menu<T>>) {
        self.sender
            .send(TrayUpdate::Menu(menu.unwrap_or_else(Menu::empty)))
            .unwrap_or_else(|err| log::warn!("Failed to send update: {err}"));
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