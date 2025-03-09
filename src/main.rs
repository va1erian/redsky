use atrium_api::app::bsky::feed::post::{self};
use atrium_api::types::TryFromUnknown;
use bsky_sdk::BskyAgent;
use tokio;
use atrium_api::types::string::Datetime;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Sender, Receiver};

#[derive(Debug)]
struct Post {
    content: String,
    author: String
}

#[derive(Debug)]
enum BskyActorMsg {
    Login {login: String, pass: String},
    Post {msg_body: String},
    GetTimeline()
}

enum BskyActorReply {
    OkReply(),
    GetTimelineReply{posts: Vec<Post>}
}

struct BskyActor {
    tx: Sender<Result<BskyActorReply, Box<dyn std::error::Error + Send>>>,
    rx: Receiver<BskyActorMsg>,
    bsky_agent: BskyAgent,
}

struct RedskyApp {
    tx: Sender<BskyActorMsg>,
    rx: Receiver<Result<BskyActorReply, Box<dyn std::error::Error + Send>>>,

    login: String,
    pass: String,
    reply: String,
    msg: String,
    timeline: Vec<Post>
}


impl RedskyApp {
    pub fn new(tx: Sender<BskyActorMsg>, rx: Receiver<Result<BskyActorReply, Box<dyn std::error::Error + Send>>>) -> Self {
        Self {
            tx,
            rx,
            login: String::new(),
            pass: String::new(),
            reply: String::new(),
            msg: String::new(),
            timeline: Vec::new()
        }
    }
}

impl BskyActor {
    pub fn new(agent: BskyAgent, rx: Receiver<BskyActorMsg>, tx: Sender<Result<BskyActorReply, Box<dyn std::error::Error + Send>>>) -> Self {
        Self {
            tx,
            rx,
            bsky_agent: agent,
        }
    }

    pub async fn listen(&mut self) -> bool {
        if let Some(msg) = self.rx.recv().await {
            match msg {
                BskyActorMsg::Login { login, pass } => {
                    let _ = self.login(login, pass).await;
                    let _ = self.tx.send(Ok(BskyActorReply::OkReply())).await;
                    true
                }
                BskyActorMsg::Post { msg_body } => {
                    let _ = self.post(msg_body).await;
                    let _ = self.tx.send(Ok(BskyActorReply::OkReply())).await;
                    true
                }
                BskyActorMsg::GetTimeline() => {
                    let result = self.get_timeline_posts().await;
                    let _ = self.tx.send(result).await;
                    true
                }            
            }
        } else {
        false
        }
    }

    pub async fn get_timeline_posts(&self) -> Result<BskyActorReply, Box<dyn std::error::Error + Send>> {
        dbg!("get tl");

        let posts = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_timeline( atrium_api::app::bsky::feed::get_timeline::ParametersData{
            algorithm: None,
            cursor: None,
            limit: None
        }.into()).await.unwrap();

        Ok(BskyActorReply::GetTimelineReply 
            { 
                posts: posts.data.feed.iter().map(|feed_element| {
                    let post 
                    = post::RecordData::try_from_unknown(feed_element.data.post.data.record.clone()).unwrap();
                Post {
                    author: feed_element.post.author.display_name.clone()
                        .or(Some("none".to_string())).unwrap(),
                    content: post.text
                }}
            ).collect()
        })
    }

    pub async fn login(&self, login: String, pass: String) -> Result<BskyActorReply, Box<dyn std::error::Error>> {
        dbg!("loggin in");
        let result = self.bsky_agent.login(login, pass).await.unwrap();
        dbg!(result);
        Ok(BskyActorReply::OkReply())
    } 

    pub async fn post(&self, msg: String) -> Result<BskyActorReply, Box<dyn std::error::Error>> {
        dbg!("post");
        let _ = self.bsky_agent.create_record(atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: msg,
        })
        .await;
        Ok(BskyActorReply::OkReply())
    }
}

impl RedskyApp {
    pub fn post_message(&mut self, msg: BskyActorMsg) -> () {
        let _ = self.tx.blocking_send(msg);
    }
}

impl eframe::App for RedskyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(Ok(msg)) = self.rx.try_recv() {
            match msg {
                BskyActorReply::OkReply() =>  {
                    dbg!("ok!");
                },
                BskyActorReply::GetTimelineReply { posts } => {
                    self.timeline = posts;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Redsky");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true),|ui| {

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&self.reply)
                    });
                    ui.horizontal(|ui| {
                        ui.text_edit_multiline(&mut self.msg );
                    });
                    ui.horizontal(|ui| {
                        let name_label = ui.label("username: ");
                        ui.text_edit_singleline(&mut self.login)
                            .labelled_by(name_label.id);
                    });
                    ui.horizontal(|ui| {
                        let name_label = ui.label("password: ");
                        ui.text_edit_singleline(&mut self.pass)
                            .labelled_by(name_label.id);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("send").clicked() {
                            self.post_message(BskyActorMsg::Login { login: self.login.to_string(), 
                                pass:self.pass.to_string() });
                            self.post_message(BskyActorMsg::Post { msg_body: self.msg.to_string() });
                        }
                        if ui.button("refresh").clicked() {
                            self.post_message(BskyActorMsg::Login { login: self.login.to_string(), 
                                pass:self.pass.to_string() });
                            self.post_message(BskyActorMsg::GetTimeline());
                        }
                    });
                });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("timeline");
                        ui.separator();

                        for post in &self.timeline {
                            ui.label(format!("{} - {}", post.author, post.content));
                            ui.separator();
                        }
                    });
                });
            });
        });
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new().expect("Unable to create Runtime");
    let (msg_tx , msg_rx) = tokio::sync::mpsc::channel(20);
    let (result_tx, result_rx) = tokio::sync::mpsc::channel(20);

    let _enter = rt.enter();
    let app = RedskyApp::new(msg_tx, result_rx);

    //spawn bsky actor
    std::thread::spawn(move || {
        let agent = match rt.block_on(BskyAgent::builder().build()) {
            Err(e) => panic!("{}", e),
            Ok(agent) => agent
        };

        let mut actor = BskyActor::new(agent, msg_rx, result_tx);
        loop {
            let should_continue = rt.block_on(actor.listen());
            if !should_continue {
                break;
            }
        }
        println!("bye");
    });

    // Run the GUI in the main thread.
    let _ = eframe::run_native(
        "Redsky",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    );
    Ok(())
}
    


