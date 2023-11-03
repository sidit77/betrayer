use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};
use parking_lot::Mutex;
use zbus::{dbus_interface, SignalContext};
use zbus::zvariant::{OwnedValue, Str, Value};
use crate::{Menu, MenuItem, TrayEvent};

struct MenuEntry<T> {
    properties: HashMap<String, OwnedValue>,
    children: Vec<usize>,
    signal: Option<T>
}

impl<T> MenuEntry<T> {
    fn get_properties(&self, requested: &[&str]) -> HashMap<String, OwnedValue> {
        self.properties
            .iter()
            .filter_map(|(k, v)| (requested.is_empty() || requested.contains(&k.as_str()))
                .then(|| (k.clone(), v.clone())))
            .collect()
    }
}

pub struct DBusMenu<T> {
    revision: AtomicU32,
    entries: Mutex<Vec<MenuEntry<T>>>,
    callback: Mutex<Box<dyn FnMut(TrayEvent<T>) + Send + 'static>>
}

impl<T> DBusMenu<T> {
    pub fn new<F>(menu: Menu<T>, callback: F) -> Self 
        where F: FnMut(TrayEvent<T>) + Send + 'static
    {
        let mut entries = Vec::new();

        entries.push(MenuEntry {
            properties: HashMap::from([(String::from("children-display"), OwnedValue::from(Str::from_static("submenu")))]),
            children: (1..(menu.items.len() + 1)).collect(),
            signal: None,
        });

        let mut items = VecDeque::from_iter(menu.items);

        while let Some(item) = items.pop_front() {
            let entry = match item {
                MenuItem::Separator => MenuEntry {
                    properties: HashMap::from([
                        (String::from("type"), OwnedValue::from(Str::from_static("separator")))
                    ]),
                    children: vec![],
                    signal: None,
                },
                MenuItem::Button { name, signal, checked } => MenuEntry {
                    properties: HashMap::from([
                        (String::from("label"), OwnedValue::from(Str::from(name))),
                        (String::from("toggle-type"), OwnedValue::from(Str::from_static("checkmark"))),
                        (String::from("toggle-state"), OwnedValue::from(if checked {1i32 } else { 0i32 }))
                    ]),
                    children: vec![],
                    signal: Some(signal),
                },
                MenuItem::Menu { name, children } => MenuEntry {
                    properties: HashMap::from([
                        (String::from("label"), OwnedValue::from(Str::from(name))),
                        (String::from("children-display"), OwnedValue::from(Str::from_static("submenu")))
                    ]),
                    children: {
                        let start = 1 + entries.len() + items.len();
                        items.extend(children);
                        let end = 1 + entries.len() + items.len();
                        (start..end).collect()
                    },
                    signal: None,
                }
            };
            entries.push(entry);
        }

        Self {
            revision: AtomicU32::new(0),
            entries: Mutex::new(entries),
            callback: Mutex::new(Box::new(callback)),
        }
    }
} 


fn collect<T>(ids: &Vec<usize>, entries: &Vec<MenuEntry<T>>, property_names: &Vec<&str>, depth: u32) -> Vec<OwnedValue> {
    match depth {
        0 => Vec::new(),
        _ => ids
            .iter()
            .copied()
            .map(|id| {
                let entry = entries.get(id).unwrap();
                Value::new((
                    id as u32,
                    entry.get_properties(property_names),
                    collect(&entry.children, entries, property_names, depth - 1))).to_owned()
            })
            .collect()
    }
}

#[dbus_interface(name = "com.canonical.dbusmenu")]
impl<T: Clone + Send + 'static> DBusMenu<T> {

    fn get_layout(&self, parent_id: i32, recursion_depth: i32, property_names: Vec<&str>) -> (u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>)) {
        log::trace!("get_layout({}, {}, {:?})", parent_id, recursion_depth, property_names);
        let depth = u32::try_from(recursion_depth)
            .unwrap_or(u32::MAX);
        let entries = self.entries.lock();
        let entry = entries.get(parent_id as usize).unwrap();
        let revision = self.revision.load(Ordering::SeqCst);
        (revision, (parent_id, entry.get_properties(&property_names), collect(&entry.children, &entries, &property_names, depth)))
    }

    fn get_group_properties(&self, ids: Vec<i32>, property_names: Vec<&str>) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        log::trace!("get_group_properties({:?}, {:?})", ids, property_names);
        let entries = self.entries.lock();
        entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| (ids.is_empty() || ids.contains(&(i as i32)))
                .then(|| (i as i32, e.get_properties(&property_names))))
            .collect()
    }

    fn get_property(&self, id: i32, name: &str) -> OwnedValue {
        log::trace!("get_property({:?}, {:?})", id, name);
        self.entries
            .lock()
            .get(id as usize)
            .and_then(|e| e.properties
                .get(name)
                .cloned())
            .unwrap()
    }

    fn event(&self, id: i32, event_id: &str, data: Value<'_>, timestamp: u32) {
        log::trace!("event({}, {}, {:?}, {})", id, event_id, data, timestamp);
        if event_id == "clicked" {
            let signal = self
                .entries
                .lock()
                .get(id as usize)
                .and_then(|e|e.signal.clone());
            if let Some(signal) = signal {
                (self.callback.lock())(TrayEvent::Menu(signal));
            }
        }
    }

    fn event_group(&self, events: Vec<(i32, &str, Value<'_>, u32)>) -> Vec<i32> {
        for (id, event, data, timestamp) in events {
            self.event(id, event, data, timestamp);
        }
        //TODO return list of failed ids
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