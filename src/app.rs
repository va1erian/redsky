use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use egui::load::Bytes;
use egui::ImageSource;
use egui::{RichText, Ui};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Post {
    content: String,
    author: String,
    display_name: String,
    date: String,
    like_count: i64,
    embeds: Vec<PostImage>
}

impl Post {
    pub fn new(content: String, author: String, display_name: String, date: String, like_count: i64, embeds: Vec<PostImage>) -> Self {
        Post {
            content,
            author,
            display_name,
            date,
            like_count,
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
    PrepareUserView{username: String},
    NotifyImageLoaded{url: String, data: Arc<[u8]>},
    RefreshTimelineMsg{posts: Vec<Post>},
    ShowUserPostsMsg{username: String, posts: Vec<Post>},
    DropUserPostsMsg{username: String},
    ShowErrorMsg{error: String}
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BskyActorMsg {
    Login {login: String, pass: String},
    Post {msg_body: String},
    GetTimeline(),
    GetUserPosts {username: String},
    LoadImage{url: String},
    Close()
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum MainViewState {
    Login,
    TimelineFeed,
    OwnPostFeed
}

pub struct RedskyApp {
    tx: Sender<BskyActorMsg>,
    ui_tx: Sender<RedskyUiMsg>,
    rx: Receiver<RedskyUiMsg>,

    is_logged_in: bool,
    is_login_window_open: bool,
    is_post_window_open: bool,
    main_view_state: MainViewState,
    login: String,
    pass: String,
    msg: String,
    timeline: Vec<Post>,
    user_posts: HashMap<String, Option<Vec<Post>>>,
    image_cache: HashMap<String, Arc<[u8]>>
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
            main_view_state: MainViewState::Login,
            login: String::new(),
            pass: String::new(),
            msg: String::new(),
            timeline: Vec::new(),
            user_posts: HashMap::new(),
            image_cache: HashMap::new()
        }
    }
}

impl RedskyApp {
    fn post_message(&self, msg: BskyActorMsg) -> () {
        let _ = self.tx.send(msg);
    }

    fn post_ui_message(&self, msg: RedskyUiMsg) -> () {
        let _ = self.ui_tx.send(msg);
    }

    fn request_post_images(&self, posts: &Vec<Post>) {
        for post in posts {
            for embed in &post.embeds {
                println!("requesting image {}", embed.thumbnail_url );
                self.post_message(BskyActorMsg::LoadImage { url: embed.thumbnail_url.clone() });
            }
        }
    }

    fn process_message(&mut self, msg: RedskyUiMsg) -> () {
        
        match msg {
            RedskyUiMsg::PostSucceeed () => {
                self.post_message(BskyActorMsg::GetTimeline());
            }
            RedskyUiMsg::RefreshTimelineMsg { posts } => {
                self.request_post_images(&posts);
                self.timeline = posts;
            }
            RedskyUiMsg::PrepareUserView { username } => {
                self.user_posts.insert(username, None);
            }
            RedskyUiMsg::ShowUserPostsMsg { username, posts }  => {
                self.request_post_images(&posts);
                self.user_posts.insert(username, Some(posts));
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
                self.main_view_state = MainViewState::OwnPostFeed;
                self.post_message(BskyActorMsg::GetUserPosts{username: self.login.clone()});
                self.post_message(BskyActorMsg::GetTimeline());
            }
            RedskyUiMsg::NotifyImageLoaded { url, data } => {
                self.image_cache.insert(url.clone(), data);
                println!("image {} loaded", url);
            }
        }
    }

    fn make_maybe_user_post_view(&self, ui: &mut Ui, username: &str, posts: &Option<Vec<Post>>) {
        match posts {
            Some(posts) => {
                self.make_post_view(ui, username, posts);
            }
            None => {
                self.make_placeholder_post_view(ui, username);
            }
        }
    }

    fn make_placeholder_post_view(&self, ui: &mut Ui, username: &str) {
        ui.vertical(|ui| {
            ui.heading(username);
            ui.separator();
                ui.spinner();
        });
    }

    fn make_post_view(&self, ui: &mut Ui, username: &str, posts: &Vec<Post>) {
        ui.vertical(|ui| {
            ui.heading(username);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui|  {
                        for post in posts {
                            ui.horizontal(|ui| {
                                if ui.label(RichText::new(&post.display_name).strong()).clicked() {
                                    self.post_ui_message(RedskyUiMsg::PrepareUserView { username: post.author.clone() });
                                    self.post_message(BskyActorMsg::GetUserPosts { username: post.author.clone() });
                                };
                                ui.label(&post.author);
                            });
                            ui.label(RichText::new(&post.date).small());
                            ui.label( &post.content);
                            if !&post.embeds.is_empty() {
                                ui.horizontal_wrapped(|ui|{
                                    ui.set_min_height(200f32);
                                    for embed in &post.embeds {
                                        if self.image_cache.contains_key(&embed.thumbnail_url) {
                                            let mut s = DefaultHasher::new();
                                            embed.thumbnail_url.hash(&mut s);
                                            let hash = s.finish();
                                            let img = self.image_cache.get(&embed.thumbnail_url).unwrap();
                                            let image_id = format!("bytes://{}.jpg", hash);
        
                                            ui.image(ImageSource::Bytes { 
                                                uri: Cow::from(image_id), 
                                                bytes: Bytes::Shared(img.clone())
                                            });
                                            
                                        } else {
                                            ui.spinner();
                                        }
                                    }
                                });
                            }
                            ui.horizontal(|ui| {
                                ui.label(format!("{} x ❤", &post.like_count));
                            });
                            ui.separator();
                        }
                    });
    
                });
        });
    }

    fn make_user_timelines_views(&self, ctx: &egui::Context) {
        for (username, posts) in self.user_posts.iter().filter(|el| *el.0 != self.login) {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(username),
                egui::ViewportBuilder::default()
                    .with_title(format!("Posts of {}", &username))
                    .with_inner_size([400.0, 600.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        self.make_maybe_user_post_view(ui, username, posts);
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.ui_tx.send(RedskyUiMsg::DropUserPostsMsg { username: username.clone() }).unwrap();
                    }
                });
        }
    }

    fn make_new_post_view(&mut self, ctx: &egui::Context) { 
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("__new_post"),
            egui::ViewportBuilder::default()
                .with_title("Post to bsky")
                .with_inner_size([200.0, 120.0]),
            |ctx, _| {
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
                    self.is_post_window_open = false;
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

                if self.main_view_state != MainViewState::Login {
                    ui.horizontal_top(|ui|{
                        ui.selectable_value(&mut self.main_view_state, MainViewState::OwnPostFeed, RichText::new("Profile").heading());
                        ui.selectable_value(&mut self.main_view_state, MainViewState::TimelineFeed, RichText::new("Timeline feed").heading());
                    });
                }
                match self.main_view_state {
                    MainViewState::Login => {
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered_justified(|ui| {
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

                        });
                    }
                    
                    MainViewState::TimelineFeed => {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true),|ui| {
                            ui.vertical(|ui| {
                                self.make_post_view(ui, "Your timeline", &self.timeline);
                            });
                        });
                    }
                    MainViewState::OwnPostFeed => {
                        match self.user_posts.get(&self.login) {
                            Some(maybe_post) => {
                                self.make_maybe_user_post_view(ui, &self.login, maybe_post);
                            }
                            None => {
                                self.make_maybe_user_post_view(ui, &self.login, &None);

                            }
                        }
                    }
                }


            })

        });
    }
}

