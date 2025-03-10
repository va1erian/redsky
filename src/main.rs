mod bsky_actor;
mod app;

use crate::app::RedskyApp;

use bsky_actor::BskyActor;
use bsky_sdk::BskyAgent;
use tokio;
use tokio::runtime::Runtime;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Run the GUI in the main thread.
    let _ = eframe::run_native(
        "Redsky",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            let (msg_tx , msg_rx) = tokio::sync::mpsc::channel(20);
            let (result_tx, result_rx) = tokio::sync::mpsc::channel(20);
        
            let _enter = rt.enter();
            let app = RedskyApp::new(msg_tx,result_tx.clone(), result_rx);
            let agent = match rt.block_on(BskyAgent::builder().build()) {
                Err(e) => panic!("{}", e),
                Ok(agent) => agent
            };

            let mut actor = BskyActor::new(agent,_cc.egui_ctx.clone(), msg_rx, result_tx);

                //spawn bsky actor
                std::thread::spawn(move || {
                
                    loop {
                        let should_continue = rt.block_on(actor.listen());
                        if !should_continue {
                            break;
                        }
                    }
                    println!("bye");
                });
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );
    Ok(())
}
    


