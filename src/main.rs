use std::collections::HashMap;

use atrium_api::app::bsky::feed::post::{self, RecordEmbedRefs};
use atrium_api::types::{TryFromUnknown, Union};
use atrium_api::xrpc::http::Response;
use bsky_sdk::BskyAgent;
use tokio;
use atrium_api::types::string::{AtIdentifier, Cid, Datetime, Did};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Sender, Receiver};

#[derive(Debug)]
struct Post {
    content: String,
    author: String,
    display_name: String,
    embeds: Vec<Blob>
}

#[derive(Clone, Debug)]
struct Blob {
    cid: String,
    did: String
}

#[derive(Debug)]
enum BskyActorMsg {
    Login {login: String, pass: String},
    Post {msg_body: String},
    GetTimeline(),
    GetBlob {blob: Blob},
    GetUserPosts {username: String}
}

enum RedskyUiMsg {
    NoOpMsg(),
    RefreshTimelineMsg{posts: Vec<Post>},
    ShowUserPostsMsg{username: String, posts: Vec<Post>},
    DropUserPostsMsg{username: String},
    LoadBlob{blob: Blob, blob_data: Vec<u8>},
    DropBlob{blob_id: String},
    ShowErrorMsg{error: String}
}

struct BskyActor {
    tx: Sender<RedskyUiMsg>,
    rx: Receiver<BskyActorMsg>,
    bsky_agent: BskyAgent,
}

struct RedskyApp {
    tx: Sender<BskyActorMsg>,
    ui_tx: Sender<RedskyUiMsg>,
    rx: Receiver<RedskyUiMsg>,

    login: String,
    pass: String,
    reply: String,
    msg: String,
    timeline: Vec<Post>,
    user_posts: HashMap<String, Vec<Post>>
}


impl RedskyApp {
    pub fn new(tx: Sender<BskyActorMsg>, 
        ui_tx: Sender<RedskyUiMsg>,
        rx: Receiver<RedskyUiMsg>) -> Self {
        Self {
            tx,
            ui_tx,
            rx,
            login: String::new(),
            pass: String::new(),
            reply: String::new(),
            msg: String::new(),
            timeline: Vec::new(),
            user_posts: HashMap::new()
        }
    }
}

impl BskyActor {
    pub fn new(agent: BskyAgent, rx: Receiver<BskyActorMsg>, tx: Sender<RedskyUiMsg>) -> Self {
        Self {
            tx,
            rx,
            bsky_agent: agent,
        }
    }

    pub async fn listen(&mut self) -> bool {
        if let Some(msg) = self.rx.recv().await {
            let result = match msg {
                BskyActorMsg::Login { login, pass } => {
                    self.login(login, pass).await
                }
                BskyActorMsg::Post { msg_body } => {
                    self.post(msg_body).await
                }
                BskyActorMsg::GetTimeline() => {
                    self.get_timeline_posts().await
                }            
                BskyActorMsg::GetUserPosts { username } => {
                    self.get_user_posts(username).await
                }
                BskyActorMsg::GetBlob { blob } => {
                    self.get_blob(blob).await
                }
            };
            if let Ok(reply) = result {
                self.tx.send(reply).await.unwrap();
            } else if let Err(e) = result {
                self.tx.send(RedskyUiMsg::ShowErrorMsg { error: e.to_string() }).await.unwrap();
            }
            true
        } else {
        false
        }
    }

    async fn get_blob(&self, blob: Blob ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send>>  {
        dbg!("get blob");
        let response = self.bsky_agent
        .api
        .com
        .atproto
        .sync
        .get_blob(atrium_api::com::atproto::sync::get_blob::ParametersData {
            cid: Cid::new(blob.cid.parse().unwrap()),
            did: Did::new(blob.did.clone()).unwrap()
        }.into()).await.unwrap();

        Ok(RedskyUiMsg::LoadBlob { blob: blob.clone(), blob_data: response})
    }

    async fn get_user_posts(&self, username: String)  -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send>> {
        dbg!("get user posts");
        let at_uri = format!("at://{}", username);
        dbg!(&at_uri);
        let response = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(atrium_api::app::bsky::feed::get_author_feed::ParametersData {
            actor: AtIdentifier::Handle(username.parse().unwrap()),
            cursor: None,
            filter: None,
            include_pins: Some(true),
            limit: 30.try_into().ok()
        }.into()).await.unwrap();

        Ok(RedskyUiMsg::ShowUserPostsMsg{
            username,
            posts: response.data.feed.iter().map(|post_el| {
                let post = post::RecordData::try_from_unknown(post_el.post.data.record.clone()).unwrap();

                Post {
                    display_name: post_el.post.author.display_name.clone()
                        .or(Some("none".to_string())).unwrap(),
                    content: post.text,
                    author: post_el.post.author.handle.to_string(),
                    embeds: vec![]
                }
        }).collect()
        })
    }

    async fn get_timeline_posts(&self) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send>> {
        dbg!("get tl");

        let posts = self.bsky_agent
        .api
        .app
        .bsky
        .feed
        .get_timeline( atrium_api::app::bsky::feed::get_timeline::ParametersData{
            algorithm: None,
            cursor: None,
            limit: 30.try_into().ok()
        }.into()).await.unwrap();

        Ok(RedskyUiMsg::RefreshTimelineMsg 
            { 
                posts: posts.data.feed.iter().map(|feed_element| {
                    let post 
                    = post::RecordData::try_from_unknown(feed_element.data.post.data.record.clone()).unwrap();
                Post {
                    display_name: feed_element.post.author.display_name.clone()
                        .or(Some("none".to_string())).unwrap(),
                    content: post.text,
                    author: feed_element.post.author.handle.to_string(),
                    embeds: vec![]
                }}
            ).collect()
        })
    }

    async fn login(&self, login: String, pass: String) -> Result<RedskyUiMsg, Box<dyn std::error::Error+ Send>> {
        dbg!("loggin in");
        let result = self.bsky_agent.login(login, pass).await.unwrap();
        dbg!(result);
        Ok(RedskyUiMsg::NoOpMsg())
    } 

    async fn post(&self, msg: String) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send>> {
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
        Ok(RedskyUiMsg::NoOpMsg())
    }
}

impl RedskyApp {
    pub fn post_message(&self, msg: BskyActorMsg) -> () {
        let _ = self.tx.try_send(msg);
    }
}

impl eframe::App for RedskyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(msg) = self.rx.try_recv() {
            match msg {
                RedskyUiMsg::NoOpMsg() =>  {},
                RedskyUiMsg::RefreshTimelineMsg { posts } => {
                    self.timeline = posts;
                }
                RedskyUiMsg::ShowUserPostsMsg { username, posts }  => {
                    self.user_posts.insert(username, posts);
                }
                RedskyUiMsg::DropUserPostsMsg { username } => {
                    self.user_posts.remove(&username);
                }
                RedskyUiMsg::ShowErrorMsg { error } => {
                    print!("error: {}", error);
                }
                RedskyUiMsg::LoadBlob { blob, blob_data } => {
                    //todo
                }
                RedskyUiMsg::DropBlob { blob_id } => {
                    //todo
                }
            }
        }

        for (username, posts) in &self.user_posts {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(username),
                egui::ViewportBuilder::default()
                    .with_title(format!("Posts of {}", &username))
                    .with_inner_size([400.0, 600.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                        egui::CentralPanel::default().show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.vertical(|ui|  {
                                    ui.heading(username);
                                    ui.separator();
                                    for post in posts {
                                        ui.label(format!("{} - {}", post.author, post.content));
                                        ui.separator();
                                    }
                                });
                            });
                        });

                        if ctx.input(|i| i.viewport().close_requested()) {
                            self.ui_tx.try_send(RedskyUiMsg::DropUserPostsMsg { username: username.clone() }).unwrap();
                        }
                });
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
                            if ui.label(format!("{} - {}", post.display_name, post.content)).clicked() {
                                self.post_message(BskyActorMsg::GetUserPosts { username: post.author.to_string() });
                            };
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
    let app = RedskyApp::new(msg_tx,result_tx.clone(), result_rx);

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
    


