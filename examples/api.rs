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

    let proxy = event_loop.create_proxy();
    let _tray = TrayIconBuilder::new()
        .with_tooltip("Demo System Tray")
        .with_menu(Menu::new([
            MenuItem::menu("Profiles", [
                MenuItem::button("Music", Signal::Profile(0)),
                MenuItem::button("Gaming", Signal::Profile(1)),
            ]),
            MenuItem::separator(),
            MenuItem::button("Open", Signal::Open),
            MenuItem::button("Quit", Signal::Quit)
        ]))
        .build(move |s| {let _ = proxy.send_event(s); });

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run(|event, evtl| {
        match event {
            Event::UserEvent(TrayEvent::Menu(signal)) => {
                log::info!("Signal: {:?}", signal);
                if signal == Signal::Quit {
                    evtl.exit();
                }
            }
            _ => {}
        }
    })?;
    Ok(())
}