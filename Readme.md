# Betrayer

Betrayer is a library for creating tray icons specifically designed to integrate well with `winit` and other existing libraries.

## Example

## Platform notes
On **Windows** and **Mac** this library uses the platform native toolkits and therefore needs a running eventloop on the same thread.

On **Linux** this library uses the [`org.kde.StatusNotifierItem`](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/) DBus api and therefore requires a Desktop Environment that supports this api. Ubuntu possibly requires the `libayatana-appindicator` package.

This library will spawn its own thread to handle DBus communication so no extra eventloop is required.

## Todo

### General
- [ ] Support for changing icons
- [ ] Standalone mode that takes control of the main thread
- [ ] More menu elements
- [ ] More options for the tray

### Windows
- [ ] Recreate the tray when the taskbar restarts

### Linux
- [ ] Support creating the tray on existing async executors to avoid spawning the thread
- [ ] Gracefully handle DBus config changes
- [ ] Use OS managed temp file to avoid leaking icons on panics?

### Mac
- [ ] Add icon support
- [ ] Generally more testing, I have no idea about Mac development and kinda freestyled this in a barely working VM.