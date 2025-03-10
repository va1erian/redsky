#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod bsky_actor;
mod app;

use crate::app::RedskyApp;

use bsky_actor::BskyActor;
use bsky_sdk::BskyAgent;
use tokio;
use tokio::runtime::Runtime;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run the GUI in the main thread.
    let _ = eframe::run_native(
        "Redsky",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            let (msg_tx , msg_rx) = std::sync::mpsc::channel();
            let (result_tx, result_rx) = std::sync::mpsc::channel();
        
            let app = RedskyApp::new(msg_tx,result_tx.clone(), result_rx);
            let actor_ctx = _cc.egui_ctx.clone();

            //spawn actor thread with tokio enabled
            std::thread::spawn(move || {
                let rt = Runtime::new().expect("Unable to create Runtime");
                let _enter = rt.enter();
                let agent = match rt.block_on(BskyAgent::builder().build()) {
                    Err(e) => panic!("{}", e),
                    Ok(agent) => agent
                };

                let mut actor = BskyActor::new(agent,actor_ctx, msg_rx, result_tx);

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
    


