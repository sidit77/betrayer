use std::collections::{HashMap, HashSet, VecDeque};
use std::mem::swap;
use std::sync::atomic::{AtomicU32, Ordering};
use parking_lot::Mutex;
use zbus::{dbus_interface, SignalContext};
use zbus::zvariant::{OwnedValue, Str, Value};
use crate::{ClickType, Menu, MenuItem, TrayEvent};
use crate::platform::linux::TrayCallback;

#[derive(Clone)]
struct MenuEntry<T> {
    properties: HashMap<String, OwnedValue>,
    children: Vec<usize>,
    signal: Option<T>
}

impl<T> MenuEntry<T> {
    fn get_properties(&self, requested: &[&str]) -> HashMap<String, OwnedValue> {
        self.properties
            .iter()
            .filter(|(k, _)| requested.is_empty() || requested.contains(&k.as_str()))
            .map(clone_inner)
            .collect()
    }
}

pub struct DBusMenu<T> {
    revision: AtomicU32,
    entries: Mutex<Vec<MenuEntry<T>>>,
    callback: TrayCallback<T>
}

impl<T> DBusMenu<T> {
    pub fn new(menu: Menu<T>, callback: TrayCallback<T>) -> Self
    {

        let entries = build_menu(menu);
        Self {
            revision: AtomicU32::new(0),
            entries: Mutex::new(entries),
            callback,
        }
    }

} 

impl<T: Clone + Send + 'static> DBusMenu<T> {
    pub async fn update_menu(&self, menu: Menu<T>, signal_context: &SignalContext<'_>) -> zbus::Result<()> {
        let (layout, updated, removed) = {
            let mut current_entries = self.entries.lock();
            let mut entries = build_menu(menu);
            swap(&mut entries, &mut current_entries);
            generate_diff(&current_entries, &entries)
        };
        if let Some(parent) = layout {
            let revision = self.revision.fetch_add(1, Ordering::SeqCst) + 1;
            log::trace!("Sending layout update signal (parent: {parent}, revision: {revision})");
            Self::layout_updated(signal_context, revision, parent).await?;
        }
        if !updated.is_empty() || !removed.is_empty() {
            log::trace!("Sending property update signal (Updated: {updated:?}, Removed: {removed:?}");
            Self::items_properties_updated(signal_context, &updated, &removed).await?;
        }
        Ok(())
    }
}

fn build_menu<T>(menu: Menu<T>) -> Vec<MenuEntry<T>> {
    log::trace!("Building layout");
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
            MenuItem::Button { name, signal, checked } => {
                let props = match checked {
                    Some(checked) => HashMap::from([
                        (String::from("label"), OwnedValue::from(Str::from(name))),
                        (String::from("toggle-type"), OwnedValue::from(Str::from_static("checkmark"))),
                        (String::from("toggle-state"), OwnedValue::from(if checked {1i32 } else { 0i32 }))
                    ]),
                    None =>HashMap::from([
                        (String::from("label"), OwnedValue::from(Str::from(name)))
                    ])
                };
             
                MenuEntry {
                    properties: props,
                    children: vec![],
                    signal: Some(signal),
                }
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
    entries
}

fn generate_diff<T>(new: &Vec<MenuEntry<T>>, old: &Vec<MenuEntry<T>>) -> (Option<i32>, Vec<(i32, HashMap<String, OwnedValue>)>, Vec<(i32, Vec<String>)>) {
    let mut updated = Vec::new();
    let mut removed = Vec::new();
    let mut changed = HashSet::new();
    for (i, (new, old)) in new.iter().zip(old.iter()).enumerate() {
        let r: Vec<String> = old.properties
            .keys()
            .filter(|k| !new.properties.contains_key(*k))
            .cloned()
            .collect();
        if !r.is_empty() {
            removed.push((i as i32, r));
        }
        let n: HashMap<String, OwnedValue> = new.properties
            .iter()
            .filter(|(k, v)| !old.properties
                .get(*k)
                .is_some_and(|ov| ov == *v))
            .map(clone_inner)
            .collect();
        if !n.is_empty() {
            updated.push((i as i32, n));
        }
        if new.children != old.children {
            changed.insert(i);
        }
    }
    let changed = match changed.len() {
        0 => None,
        1 => Some(*changed.iter().next().expect("There should be one element here") as i32),
        _ => Some(find_common_root(new, &changed) as i32)
    };
    (changed, updated, removed)
}

fn find_common_root<T>(entries: &Vec<MenuEntry<T>>, changed: &HashSet<usize>) -> usize {
    let mut cache = HashMap::new();
    for (i, entry) in entries.iter().enumerate().rev() {
        let c: u32 = u32::from(changed.contains(&i)) + entry
            .children
            .iter()
            .map(|i| cache
                .get(i)
                .expect("The is now in breadth first order"))
            .sum::<u32>();
        cache.insert(i, c);
    }
    cache
        .iter()
        .filter_map(|(k, v)| (*v > 1).then_some(*k))
        .min()
        .expect("There should be a common root")
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
        match event_id {
            "clicked" => {
                let signal = self
                    .entries
                    .lock()
                    .get(id as usize)
                    .and_then(|e|e.signal.clone());
                if let Some(signal) = signal {
                    (self.callback.lock())(TrayEvent::Menu(signal));
                }
            },
            "opened" if id == 0 => {
                (self.callback.lock())(TrayEvent::Tray(ClickType::Left));
            }
            _ => {}
        }
    }

    fn event_group(&self, events: Vec<(i32, &str, Value<'_>, u32)>) -> Vec<i32> {
        for (id, event, data, timestamp) in events {
            self.event(id, event, data, timestamp);
        }
        //TODO return list of failed ids
        Vec::new()
    }

    fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, _ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        Default::default()
    }


    #[dbus_interface(signal)]
    async fn item_activation_requested(ctx: &SignalContext<'_>, id: i32, timestamp: u32) -> zbus::Result<()> { }

    #[dbus_interface(signal)]
    async fn items_properties_updated(ctx: &SignalContext<'_>, updated_props: &[(i32, HashMap<String, OwnedValue>, )], removed_props: &[(i32, Vec<String>)]) -> zbus::Result<()> { }

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

fn clone_inner<T1: Clone, T2: Clone>((a, b): (&T1, &T2)) -> (T1, T2) {
    (a.clone(), b.clone())
}