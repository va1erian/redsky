#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod bsky_actor;
mod updater;

use crate::app::RedskyApp;

use bsky_actor::BskyActor;
use bsky_sdk::BskyAgent;
use tokio;
use tokio::runtime::Runtime;

fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(include_bytes!("../resources/redsky.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut is_screenshot_mode = false;
    let mut screenshot_output_path = None;

    for arg in std::env::args() {
        if arg == "--test-screenshot" {
            is_screenshot_mode = true;
        } else if arg.starts_with("--test-screenshot-output=") {
            screenshot_output_path = Some(arg.trim_start_matches("--test-screenshot-output=").to_string());
        }
    }

    let mut options = eframe::NativeOptions::default();
    options.viewport = egui::ViewportBuilder::default().with_icon(std::sync::Arc::new(load_icon()));

    // Run the GUI in the main thread.
    let _ = eframe::run_native(
        "Redsky",
        options,
        Box::new(move |_cc| {
            let (msg_tx, msg_rx) = std::sync::mpsc::channel();
            let (result_tx, result_rx) = std::sync::mpsc::channel();

            let app = RedskyApp::new(msg_tx, result_tx.clone(), result_rx, is_screenshot_mode, screenshot_output_path);

            #[cfg(target_os = "windows")]
            {
                std::thread::spawn(|| {
                    let rt = Runtime::new().expect("Unable to create tokio runtime for updater");
                    rt.block_on(async {
                        updater::check_and_update().await;
                    });
                });
            }
            let actor_ctx = _cc.egui_ctx.clone();

            match app.settings.theme {
                crate::app::AppTheme::System => {}
                crate::app::AppTheme::Light => {
                    _cc.egui_ctx.set_visuals(egui::Visuals::light());
                }
                crate::app::AppTheme::Dark => {
                    _cc.egui_ctx.set_visuals(egui::Visuals::dark());
                }
            }

            //spawn actor thread with tokio enabled
            std::thread::spawn(move || {
                let rt = Runtime::new().expect("Unable to create tokio runtime");
                let _enter = rt.enter();
                let agent = match rt.block_on(BskyAgent::builder().build()) {
                    Err(e) => panic!("{}", e),
                    Ok(agent) => agent,
                };

                let mut actor = BskyActor::new(agent, actor_ctx, msg_rx, result_tx);

                loop {
                    if !actor.pump() {
                        break;
                    }
                }
                println!("bsky actor: bye");
            });
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );
    Ok(())
}
