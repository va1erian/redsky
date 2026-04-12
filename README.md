# Redsky

Redsky is a desktop client for [Bluesky](https://bsky.app) written in Rust using the [egui](https://github.com/emilk/egui) / eframe framework.

## Architecture

The application follows an **actor pattern** to separate the user interface from network operations:
- **`RedskyApp` (UI):** Runs on the main thread and manages the UI state using `egui`/`eframe`.
- **`BskyActor` (Background):** Runs on a separate thread with a `tokio` asynchronous runtime. It handles all network requests, image downloading, and interactions with the AT Protocol via `atrium-api`.
- The two components communicate asynchronously using `std::sync::mpsc` channels, preventing slow network operations from blocking the UI.

## Building from source

You will need the Rust toolchain installed. You can get it from [rustup.rs](https://rustup.rs).

### Windows and macOS
No extra system dependencies are typically required. Simply clone the repository and run:
```bash
cargo build --release
```
The executable will be in `target/release/`.

### Linux
On Linux, Redsky requires several system dependencies to build correctly (due to Wayland/X11, EGL, and dbus requirements).

For Ubuntu/Debian-based distributions, install the following packages:
```bash
sudo apt-get update
sudo apt-get install -y libwayland-dev libxkbcommon-dev libx11-dev libegl1-mesa-dev libgles2-mesa-dev libdbus-1-dev pkg-config
```

Then, you can build the application:
```bash
cargo build --release
```
The executable will be in `target/release/`.
