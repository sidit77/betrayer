use log::LevelFilter;
use simple_logger::SimpleLogger;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use anyhow::Result;
use betrayer::{Menu, MenuItem, TrayEvent, TrayIconBuilder};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Signal {
    Profile(u32),
    Open,
    Quit
}

fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()?;

    let event_loop = EventLoopBuilder::with_user_event()
        .build()?;

    let mut selected = 0;

    let proxy = event_loop.create_proxy();
    let tray = TrayIconBuilder::new()
        .with_tooltip("Demo System Tray")
        .with_menu(build_menu(selected))
        .build(move |s| {let _ = proxy.send_event(s); })?;

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run(|event, evtl| {
        match event {
            Event::UserEvent(TrayEvent::Menu(signal)) => {
                log::info!("Signal: {:?}", signal);
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
            _ => {}
        }
    })?;
    Ok(())
}

fn build_menu(selected: u32) -> Menu<Signal> {
    Menu::new([
        MenuItem::menu("Profiles", (0..5)
            .map(|i| MenuItem::button(format!("Profile {}", i + 1), Signal::Profile(i),selected == i))),
        MenuItem::separator(),
        MenuItem::button("Open", Signal::Open, false),
        MenuItem::button("Quit", Signal::Quit, false)
    ])
}