use winit::event_loop::EventLoop;
use crate::{TrayEvent, TrayIcon, TrayIconBuilder, TrayResult};

pub trait WinitTrayIconBuilderExt<T> {
    fn build_event_loop<E, F>(self, event_loop: &EventLoop<E>, filter_map: F) -> TrayResult<TrayIcon<T>>
        where
            F: Fn(TrayEvent<T>) -> Option<E> + Send + 'static,
            E: Send;
}

impl<T: Clone + Send + 'static> WinitTrayIconBuilderExt<T> for TrayIconBuilder<T> {
    fn build_event_loop<E, F>(self, event_loop: &EventLoop<E>, filter_map: F) -> TrayResult<TrayIcon<T>>
        where
            F: Fn(TrayEvent<T>) -> Option<E> + Send + 'static,
            E: Send
    {
        let proxy = event_loop.create_proxy();
        self.build(move |event| {
            if let Some(event) = filter_map(event) {
                proxy
                    .send_event(event)
                    .unwrap_or_else(|err| log::warn!("Failed to forward event: {}", err));
            }
        })
    }
}