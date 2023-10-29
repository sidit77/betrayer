use std::sync::mpsc::channel;
use system_status_bar_macos::*;

fn main() {
    let _status_item = StatusItem::new("TITLE", Menu::new(vec![
        MenuItem::new("UNCLICKABLE MENU", None, None),
        MenuItem::new("CLICKABLE MENU", Some(Box::new(|| {
            println!("clicked!");
        })), None),
        MenuItem::new("PARENT MENU", None, Some(Menu::new(vec![
            MenuItem::new("SUBMENU", None, None),
            MenuItem::new("SUBMENU", None, None),
        ]))),
    ]));

    let (_sender, receiver) = channel::<()>();
    sync_infinite_event_loop(receiver, |_| { });
}