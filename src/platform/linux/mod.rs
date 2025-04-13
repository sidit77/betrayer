mod item;
mod menu;

use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

use flume::Sender;
use futures_util::{StreamExt, TryStreamExt};
use parking_lot::Mutex;
use png::{BitDepth, ColorType, Encoder};
use zbus::{connection, proxy, Task};

use crate::error::{ErrorSource, TrayResult};
use crate::platform::linux::item::StatusNotifierItem;
use crate::platform::linux::menu::DBusMenu;
use crate::{Icon, Menu, TrayEvent, TrayIconBuilder};

static MENU_PATH: &str = "/MenuBar";
static ITEM_PATH: &str = "/StatusNotifierItem";
static COUNTER: AtomicU32 = AtomicU32::new(1);

enum TrayUpdate<T> {
    Menu(Menu<T>),
    Tooltip(String),
    Icon(String),
}

pub type TrayCallback<T> = Arc<Mutex<dyn FnMut(TrayEvent<T>) + Send + 'static>>;

pub struct NativeTrayIcon<T> {
    id: (u32, u32),
    sender: Sender<TrayUpdate<T>>,
    tmp_icon_file: Cell<Option<TmpFileRaiiHandle>>,
    tmp_icon_counter: Cell<u32>,
    _update_task: Task<()>,
    _register_task: Task<Result<(), zbus::Error>>,
}

impl<T: Clone + Send + 'static> NativeTrayIcon<T> {
    pub async fn new_async<F>(builder: TrayIconBuilder<T>, callback: F) -> TrayResult<Self>
    where
        F: FnMut(TrayEvent<T>) + Send + 'static,
    {
        let pid = std::process::id();
        let id = COUNTER.fetch_add(1, Ordering::AcqRel);
        let name = format!("org.kde.StatusNotifierItem-{pid}-{id}");

        let mut tmp_icon_counter = 0;
        let (icon, tmp_icon_path) = builder
            .icon
            .map(NativeIcon::from)
            .map(|icon| icon.write_to_disk((pid, id), &mut tmp_icon_counter))
            .unzip();

        let callback = Arc::new(Mutex::new(callback));
        //"/home/simon/headset-controller/resources/icon.png"
        let conn = connection::Builder::session()?
            .name(name.clone())?
            .serve_at(
                ITEM_PATH,
                StatusNotifierItem::new(icon.unwrap_or_default(), builder.tooltip.unwrap_or_default(), callback.clone()),
            )?
            .serve_at(MENU_PATH, DBusMenu::new(builder.menu.unwrap_or_else(Menu::empty), callback))?
            .internal_executor(true)
            .build()
            .await?;

        let (sender, receiver) = flume::unbounded();
        let receiver_task = {
            let connection = conn.clone();
            conn.executor().spawn(
                async move {
                    while let Ok(event) = receiver.recv_async().await {
                        match event {
                            TrayUpdate::Menu(menu) => {
                                let iface = connection
                                    .object_server()
                                    .interface::<_, DBusMenu<T>>(MENU_PATH)
                                    .await
                                    .unwrap();
                                let iref = iface.get().await;
                                iref.update_menu(menu, iface.signal_emitter())
                                    .await
                                    .unwrap();
                            }
                            TrayUpdate::Tooltip(tooltip) => {
                                let iface = connection
                                    .object_server()
                                    .interface::<_, StatusNotifierItem<T>>(ITEM_PATH)
                                    .await
                                    .unwrap();
                                let iref = iface.get().await;
                                iref.update_tooltip(tooltip, iface.signal_emitter())
                                    .await
                                    .unwrap();
                            }
                            TrayUpdate::Icon(icon) => {
                                let iface = connection
                                    .object_server()
                                    .interface::<_, StatusNotifierItem<T>>(ITEM_PATH)
                                    .await
                                    .unwrap();
                                let iref = iface.get().await;
                                iref.update_icon(icon, iface.signal_emitter())
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                },
                "event receiver",
            )
        };

        let proxy = StatusNotifierWatcherProxy::builder(&conn)
            .path("/StatusNotifierWatcher")?
            .build()
            .await?;

        proxy.register_status_notifier_item(&name).await?;

        let register_task = {
            conn.executor().spawn(
                async move {
                    proxy
                        .inner()
                        .receive_owner_changed()
                        .await?
                        .then(|new_owner| {
                            let proxy = &proxy;
                            let name = &name;
                            async move {
                                match new_owner {
                                    Some(_) => proxy.register_status_notifier_item(&name).await,
                                    None => Ok(()),
                                }
                            }
                        })
                        .try_collect::<()>()
                        .await
                },
                "statusnotifierwatcher watcher",
            )
        };

        Ok(Self {
            id: (pid, id),
            sender,
            tmp_icon_file: Cell::new(tmp_icon_path.flatten()),
            tmp_icon_counter: Cell::new(tmp_icon_counter),
            _update_task: receiver_task,
            _register_task: register_task,
        })
    }

    pub fn new<F>(builder: TrayIconBuilder<T>, callback: F) -> TrayResult<Self>
    where
        F: FnMut(TrayEvent<T>) + Send + 'static,
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

    pub fn set_icon(&self, icon: Option<Icon>) {
        let mut counter = self.tmp_icon_counter.get();
        let (icon, tmp_icon_path) = icon
            .map(NativeIcon::from)
            .map(|icon| icon.write_to_disk(self.id, &mut counter))
            .unzip();
        self.tmp_icon_counter.set(counter);
        self.tmp_icon_file.set(tmp_icon_path.flatten());
        self.sender
            .send(TrayUpdate::Icon(icon.unwrap_or_default()))
            .unwrap_or_else(|err| log::warn!("Failed to send update: {err}"));
    }
}

#[proxy(interface = "org.kde.StatusNotifierWatcher", assume_defaults = true)]
trait StatusNotifierWatcher {
    fn register_status_notifier_host(&self, service: &str) -> zbus::Result<()>;

    fn register_status_notifier_item(&self, service: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    fn status_notifier_host_registered(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn status_notifier_host_unregistered(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn status_notifier_item_registered(&self, arg_1: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    fn status_notifier_item_unregistered(&self, arg_1: &str) -> zbus::Result<()>;

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn protocol_version(&self) -> zbus::Result<i32>;

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> zbus::Result<Vec<String>>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NativeIcon {
    #[allow(dead_code)]
    Path(String),
    Pixels(Vec<u8>),
}

impl NativeIcon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> TrayResult<Self> {
        let mut pixels = Vec::new();
        let mut encoder = Encoder::new(&mut pixels, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&rgba).unwrap();
        writer.finish().unwrap();
        Ok(Self::Pixels(pixels))
    }

    pub fn from_png_bytes(bytes: &[u8]) -> TrayResult<Self> {
        Ok(Self::Pixels(bytes.to_vec()))
    }
    fn write_to_disk(&self, id: (u32, u32), counter: &mut u32) -> (String, Option<TmpFileRaiiHandle>) {
        match self {
            NativeIcon::Path(path) => (path.clone(), None),
            NativeIcon::Pixels(pixels) => {
                let tmp_path = get_tmp_icon_path(id, *counter);
                *counter += 1;
                //std::fs::create_dir_all(&tmp_path).unwrap();
                std::fs::write(&tmp_path, pixels).unwrap();
                (tmp_path.clone(), Some(TmpFileRaiiHandle(tmp_path)))
            }
        }
    }
}

struct TmpFileRaiiHandle(String);

impl Drop for TmpFileRaiiHandle {
    fn drop(&mut self) {
        let path = self.0.as_str();
        std::fs::remove_file(path).unwrap_or_else(|err| log::warn!("Failed to clean up icon file at {path}: {err}"));
    }
}

fn get_tmp_icon_path((pid, id): (u32, u32), counter: u32) -> String {
    static BASE_DIR: OnceLock<String> = OnceLock::new();

    let base = BASE_DIR.get_or_init(|| {
        let base = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .ok()
            .unwrap_or_else(std::env::temp_dir)
            .join("betrayer");
        std::fs::create_dir_all(&base).expect("Failed to create icon tmp dir");
        log::trace!("Using {base:?} as tmp dir for icons");
        base.to_str()
            .expect("Non UTF-8 paths are currently not supported")
            .to_string()
    });
    format!("{base}/icon-{pid}-{id}-{counter}.png")
}

pub type PlatformError = zbus::Error;
impl From<PlatformError> for ErrorSource {
    fn from(value: PlatformError) -> Self {
        ErrorSource::Os(value)
    }
}
