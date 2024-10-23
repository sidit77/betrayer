use anyhow::Result;
use betrayer::winit::WinitTrayIconBuilderExt;
use betrayer::{Icon, Menu, MenuItem, TrayEvent, TrayIcon, TrayIconBuilder};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Signal {
    Profile(u32),
    Open,
    Quit
}

fn main() -> Result<()> {
    SimpleLogger::new()
        .with_module_level("betrayer", LevelFilter::Trace)
        .with_level(LevelFilter::Debug)
        .init()?;

    let event_loop = EventLoop::with_user_event().build()?;

    let selected = 0;
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
    event_loop.run_app(&mut App { tray, selected })?;
    Ok(())
}

struct App {
    tray: TrayIcon<Signal>,
    selected: u32
}

impl ApplicationHandler<TrayEvent<Signal>> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: TrayEvent<Signal>) {
        log::info!("tray event: {:?}", event);
        if let TrayEvent::Menu(signal) = event {
            match signal {
                Signal::Profile(i) => {
                    if self.selected != i {
                        self.selected = i;
                        self.tray
                            .set_tooltip(format!("Active Profile: {}", self.selected));
                        self.tray.set_menu(build_menu(self.selected));
                    }
                }
                Signal::Open => {}
                Signal::Quit => event_loop.exit()
            }
        }
    }
    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, _event: WindowEvent) {}
}

fn build_menu(selected: u32) -> Menu<Signal> {
    Menu::new([
        MenuItem::menu(
            "Profiles",
            (0..5).map(|i| MenuItem::check_button(format!("Profile {}", i + 1), Signal::Profile(i), selected == i))
        ),
        MenuItem::separator(),
        MenuItem::button("Open", Signal::Open),
        MenuItem::button("Quit", Signal::Quit)
    ])
}
