use std::collections::HashMap;

use egui::{RichText, Ui};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct Post {
    content: String,
    author: String,
    display_name: String,
    embeds: Vec<PostImage>
}

impl Post {
    pub fn new(content: String, author: String, display_name: String, embeds: Vec<PostImage>) -> Self {
        Post {
            content,
            author,
            display_name,
            embeds
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct PostImage {
    thumbnail_url: String,
    url: String,
    alt: String
}

impl PostImage {
    pub fn new(thumb: String, url: String, alt: String) -> Self {
        PostImage {
            thumbnail_url: thumb,
            url,
            alt
        }
    }
}

pub enum RedskyUiMsg {
    LogInSucceededMsg(),
    PostSucceeed(),
    RefreshTimelineMsg{posts: Vec<Post>},
    ShowUserPostsMsg{username: String, posts: Vec<Post>},
    DropUserPostsMsg{username: String},
    ShowErrorMsg{error: String}
}

#[derive(Debug)]
pub enum BskyActorMsg {
    Login {login: String, pass: String},
    Post {msg_body: String},
    GetTimeline(),
    GetUserPosts {username: String}
}

pub struct RedskyApp {
    tx: Sender<BskyActorMsg>,
    ui_tx: Sender<RedskyUiMsg>,
    rx: Receiver<RedskyUiMsg>,

    is_logged_in: bool,
    is_login_window_open: bool,
    is_post_window_open: bool,
    login: String,
    pass: String,
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
            is_logged_in: false,
            is_login_window_open: true,
            is_post_window_open: false,
            login: String::new(),
            pass: String::new(),
            msg: String::new(),
            timeline: Vec::new(),
            user_posts: HashMap::new(),
        }
    }
}

impl RedskyApp {
    pub fn post_message(&self, msg: BskyActorMsg) -> () {
        let _ = self.tx.try_send(msg);
    }

    pub fn process_message(&mut self, msg: RedskyUiMsg) -> () {
        match msg {
            RedskyUiMsg::PostSucceeed () => {
                self.post_message(BskyActorMsg::GetTimeline());
            }
            RedskyUiMsg::RefreshTimelineMsg { posts } => {
                dbg!(&posts);
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
            RedskyUiMsg::LogInSucceededMsg() => {
                self.is_login_window_open = false;
                self.is_logged_in = true;
                self.post_message(BskyActorMsg::GetTimeline());
            }
        }
    }

    pub fn make_post_view(&self, ui: &mut Ui, username: &str, posts: &Vec<Post>) {
        ui.vertical(|ui| {
            ui.heading(username);
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui|  {
                        for post in posts {
                            ui.horizontal(|ui| {
                                if ui.label(RichText::new(&post.display_name).strong()).clicked() {
                                    self.post_message(BskyActorMsg::GetUserPosts { username: post.author.clone() });
                                };
                                ui.label(&post.author);
                            });
                            ui.label( &post.content);
                            for embed in &post.embeds {
                                ui.image(embed.thumbnail_url.clone());
                            }
                            ui.separator();
                        }
                    });
    
                });
        });
    }

    pub fn make_user_timelines_views(&self, ctx: &egui::Context) {
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
                            self.make_post_view(ui, username, posts);
                        });

                        if ctx.input(|i| i.viewport().close_requested()) {
                            self.ui_tx.try_send(RedskyUiMsg::DropUserPostsMsg { username: username.clone() }).unwrap();
                        }
                });
        }
    }

    pub fn make_login_window_view(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("__login"),
            egui::ViewportBuilder::default()
                .with_title("Login to Bluesky")
                .with_inner_size([200.0, 150.0]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Welcome to Redsky");
                    ui.horizontal(|ui| {
                        let name_label = ui.label("bsky handle: ");
                        ui.text_edit_singleline(&mut self.login)
                            .labelled_by(name_label.id);
                    });
                    ui.horizontal(|ui| {
                        let pwd_label = ui.label("password: ");
                        ui.add(egui::TextEdit::singleline(&mut self.pass).password(true))
                            .labelled_by(pwd_label.id);
                    });
                    ui.horizontal(|ui| {

                        if ui.button("login").clicked() {
                            self.post_message(BskyActorMsg::Login { login: self.login.to_string(), 
                                pass:self.pass.to_string() });
                        }
                    }); 
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.is_post_window_open = false;
                }
            });
    }

    pub fn make_new_post_view(&mut self, ctx: &egui::Context) { 
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("__new_post"),
            egui::ViewportBuilder::default()
                .with_title("Post to bsky")
                .with_inner_size([200.0, 120.0]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("New post");

                        ui.text_edit_multiline(&mut self.msg );
                        ui.horizontal(|ui| {
                            if ui.button("send").clicked() {
                                self.post_message(BskyActorMsg::Post { msg_body: self.msg.clone() });
                                self.msg.clear();
                            }
                        });
                    });
            
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.is_login_window_open = false;
                }
            });

    }   
}

impl eframe::App for RedskyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(msg) = self.rx.try_recv() {
            self.process_message(msg);
        }
        
        self.make_user_timelines_views(ctx);
        if self.is_login_window_open {
            self.make_login_window_view(ctx);
        }

        if self.is_post_window_open {
            self.make_new_post_view(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui | {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Login...").clicked() {
                            self.is_login_window_open = true;
                        }
                        if ui.button("New post...").clicked() {
                            self.is_post_window_open = true;
                        }
                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });
    
                });

                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true),|ui| {
                    ui.vertical(|ui| {
                        self.make_post_view(ui, "Your timeline", &self.timeline);
                });
            });
            })

        });


    }
}

