# Betrayer

Betrayer is a library for creating tray icons specifically designed to integrate well with `winit` and other existing libraries.

## Example
```rust
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Signal {
    Profile(u32),
    Open,
    Quit
}

fn main() -> Result<()> {
    let event_loop = EventLoopBuilder::with_user_event()
        .build()?;

    let mut selected = 0;
    
    let tray = TrayIconBuilder::new()
        .with_icon(Icon::from_rgba(vec![255u8; 32 * 32 * 4], 32, 32)?)
        .with_tooltip("Demo System Tray")
        .with_menu(build_menu(selected))
        // with `winit` feature:
        .build_event_loop(&event_loop, |e| Some(e))?;
        // without:
        //.build({
        //    let proxy = event_loop.create_proxy();
        //    move |s| {let _ = proxy.send_event(s); }
        //})?;

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run(|event, evtl| {
        match event {
            Event::UserEvent(event) => {
                println!("tray event: {:?}", event);
                if let TrayEvent::Menu(signal) = event {
                    match signal {
                        Signal::Profile(i) => {
                            if selected != i {
                                selected = i;
                                tray.set_tooltip(format!("Active Profile: {selected}"));
                                tray.set_menu(build_menu(selected));
                            }
                        },
                        Signal::Open => {}
                        Signal::Quit => evtl.exit()
                    }
                }
            }
            _ => {}
        }
    })?;
    Ok(())
}

fn build_menu(selected: u32) -> Menu<Signal> {
    Menu::new([
        MenuItem::menu("Profiles", (0..5)
            .map(|i| MenuItem::check_button(format!("Profile {}", i + 1), Signal::Profile(i),selected == i))),
        MenuItem::separator(),
        MenuItem::button("Open", Signal::Open),
        MenuItem::button("Quit", Signal::Quit)
    ])
}
```


## Platform notes
On **Windows** and **Mac** this library uses the platform native toolkits and therefore needs a running eventloop on the same thread.

On **Linux** this library uses the [`org.kde.StatusNotifierItem`](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/) DBus api and therefore requires a Desktop Environment that supports this api. Ubuntu possibly requires the `libayatana-appindicator` package.

This library will spawn its own thread to handle DBus communication so no extra eventloop is required.

## Todo

### General
- [ ] Standalone mode that takes control of the main thread
- [ ] More menu elements
- [ ] More options for the tray


### Linux
- [ ] Support creating the tray on existing async executors to avoid spawning the thread
- [ ] Gracefully handle DBus config changes
- [ ] Use OS managed temp file to avoid leaking icons on panics?

### Mac
- [ ] Add icon support
- [ ] Generally more testing, I have no idea about Mac development and kinda freestyled this in a barely working VM.