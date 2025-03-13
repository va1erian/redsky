use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::iter;
use std::sync::Arc;
use atrium_api::types::string::Cid;
use egui::load::Bytes;
use egui::{vec2, Context, ImageSource, Sense};
use egui::{RichText, Ui};
use egui_extras::{Size, StripBuilder};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct StrongRef {
    pub uri: String,
    pub cid: Cid
}

#[derive(Debug)]
pub struct Post {
    pub uri: String,
    pub cid: Cid,
    pub content: String,
    pub author: String,
    pub display_name: String,
    pub avatar_img: String,
    pub date: String,
    pub like_count: i64,
    pub repost_count: i64,
    pub embeds: Vec<PostImage>
}

pub struct UserProfile {
    pub handle: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_uri: String,
    pub follower_count: i64,
    pub follow_count: i64,
    pub post_count: i64
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
    PrepareThreadView{thread_ref: StrongRef},
    CloseThreadView{thread_ref: StrongRef},
    NotifyImageLoaded{url: String, data: Arc<[u8]>},
    NotifyLikesLoaded {post_uri: StrongRef, likers: Vec<UserProfile> },
    NotifyPostAndRepliesLoaded {post: Post, replies : Vec<Post>},
    ShowUserProfile{profile: UserProfile},
    RefreshTimelineMsg{posts: Vec<Post>},
    ShowUserPostsMsg{username: String, posts: Vec<Post>},
    DropUserPostsMsg{username: String},
    PrepareImageView {img_uri: String},
    ShowBigImageView {img_uri: String},
    CloseBigImageView {img_uri: String}, 
    ShowErrorMsg{error: String}
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BskyActorMsg {
    Login {login: String, pass: String},
    Post {msg_body: String},
    GetTimeline(),
    GetPostLikers {post_ref: StrongRef},
    GetPostAndReplies {post_ref: StrongRef},
    GetUserProfile{username: String},
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
    is_post_window_open: bool,
    main_view_state: MainViewState,
    login: String,
    pass: String,
    msg: String,
    timeline: Vec<Post>,
    user_posts: HashMap<String, Option<Vec<Post>>>,
    
    user_infos_cache: HashMap<String, UserProfile>,
    image_cache: HashMap<String, Option<Arc<[u8]>>>,
    post_likers_cache: HashMap<StrongRef, Vec<UserProfile>>,
    post_replies_cache: HashMap<StrongRef, Option<Vec<Post>>>,
    opened_image_views: HashSet<String>
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
            is_post_window_open: false,
            main_view_state: MainViewState::Login,
            login: String::new(),
            pass: String::new(),
            msg: String::new(),
            timeline: Vec::new(),
            user_posts: HashMap::new(),
            user_infos_cache: HashMap::new(),
            image_cache: HashMap::new(),
            post_likers_cache: HashMap::new(),
            post_replies_cache: HashMap::new(),
            opened_image_views: HashSet::new()
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

    fn request_post_images(&mut self, posts: &Vec<Post>) {
        for post in posts {
            println!("requesting avatar {}", &post.avatar_img );
            self.image_cache.insert(post.avatar_img.clone(), None);
            self.post_message(BskyActorMsg::LoadImage { url: post.avatar_img.clone()});

            for embed in &post.embeds {
                println!("requesting image {}", &embed.thumbnail_url );
                self.image_cache.insert(embed.thumbnail_url.clone(), None);
                self.post_message(BskyActorMsg::LoadImage { url: embed.thumbnail_url.clone()});
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
                self.user_posts.insert(username.clone(), None);
                self.post_message(BskyActorMsg::GetUserProfile { username });
            }

            RedskyUiMsg::PrepareImageView { img_uri } => {
                self.image_cache.insert(img_uri.clone(), None);
                self.post_message(BskyActorMsg::LoadImage { url: img_uri });
            }

            RedskyUiMsg::ShowUserProfile { profile } => {
                self.user_infos_cache.insert(profile.handle.clone(), profile.into());

            }
            RedskyUiMsg::ShowUserPostsMsg { username, posts }  => {
                self.request_post_images(&posts);
                self.user_posts.insert(username, Some(posts));
            }

            RedskyUiMsg::PrepareThreadView { thread_ref } => {
                self.post_replies_cache.insert(thread_ref.clone(), None);
                self.post_message(BskyActorMsg::GetPostAndReplies { post_ref: thread_ref });
            }

            RedskyUiMsg::CloseThreadView { thread_ref } => {
                self.post_replies_cache.remove(&thread_ref);
            }

            RedskyUiMsg::DropUserPostsMsg { username } => {
                self.user_posts.remove(&username);
            }
            RedskyUiMsg::ShowErrorMsg { error } => {
                print!("error: {}", error);
            }
            RedskyUiMsg::NotifyLikesLoaded { post_uri, likers } => {
                self.post_likers_cache.insert(post_uri, likers);
            }
            RedskyUiMsg::NotifyPostAndRepliesLoaded { post, mut replies } => {
                let strong_ref = StrongRef { uri: post.uri.clone(), cid: post.cid.clone()};
                replies.insert(0, post);
                self.request_post_images(&replies);
                self.post_replies_cache.insert(strong_ref, Some(replies));
            }
            RedskyUiMsg::LogInSucceededMsg() => {
                self.is_logged_in = true;
                self.main_view_state = MainViewState::OwnPostFeed;
                self.post_message(BskyActorMsg::GetUserPosts{username: self.login.clone()});
                self.post_message(BskyActorMsg::GetUserProfile { username: self.login.clone() });
                self.post_message(BskyActorMsg::GetTimeline());
            }
            RedskyUiMsg::NotifyImageLoaded { url, data } => {
                self.image_cache.insert(url.clone(), Some(data));
                println!("image {} loaded", url);
            }
            RedskyUiMsg::ShowBigImageView { img_uri }  => {
                self.opened_image_views.insert(img_uri.clone());
                self.post_message(BskyActorMsg::LoadImage { url: img_uri });
            }
            RedskyUiMsg::CloseBigImageView { img_uri } => {
                self.opened_image_views.remove(&img_uri);
            }
        }
    }

    fn make_maybe_user_profile_view(&self, ui: &mut Ui, username: &str, maybe_profile: Option<&UserProfile>) {
        match maybe_profile {
            Some(profile) => {
                ui.horizontal(|ui| {
                    ui.set_max_height(100f32);
                    match self.image_cache.get(&profile.avatar_uri) {
                        Some(Some(img)) => {
                            let mut s = DefaultHasher::new();
                            profile.avatar_uri.hash(&mut s);
                            let hash = s.finish();
                            let image_id = format!("bytes://{}.jpg", hash);

                                ui.vertical(|ui| {
                                    ui.set_max_width(100f32);
                                    ui.set_max_height(100f32);    
                                    ui.image(ImageSource::Bytes { 
                                        uri: Cow::from(image_id), 
                                        bytes: Bytes::Shared(img.clone())
                                    });
                                });
                        }
                        Some(None) => {
                            ui.vertical(|ui| {
                                ui.set_max_width(100f32);
                                ui.set_max_height(100f32);    
                                ui.spinner();
                            });
                        }
                        None => {
                            self.post_message(BskyActorMsg::LoadImage { url: profile.avatar_uri.clone() });
                        }
                    }
                    ui.vertical(|ui|{
                        ui.set_max_height(100f32);
                        ui.heading(&profile.display_name);
                        ui.small(&profile.handle);
                        ui.label(&profile.bio);
                        ui.label(format!("{} post(s), {} follower(s), {} follow(s)",
                            &profile.post_count, &profile.follower_count, &profile.follow_count));
                    });
                    ui.allocate_space(ui.available_size());
                });
            }
            None => {
                ui.label(username);
                ui.spinner();
            }
        }
    }

    fn make_maybe_user_post_view(&self, ui: &mut Ui, username: &str, posts: &Option<Vec<Post>>) {
        StripBuilder::new(ui)
        .size(Size::exact(100.0))
        .size(Size::remainder())
        .vertical(|mut strip| {
            strip.cell(|ui| {
                self.make_maybe_user_profile_view(ui, username, self.user_infos_cache.get(username));
                ui.separator();
            });
            strip.strip(|builder| {
                builder.sizes(Size::remainder(), 1).horizontal(|mut strip| {
                    strip.cell(|ui| {
                        match posts {
                            Some(posts) => {
                                self.make_post_view(ui, username, posts);
                            }
                            None => {
                                self.make_placeholder_post_view(ui, username);
                            }
                        }
                    }); 
                });
            });
        });
    }

    fn make_placeholder_post_view(&self, ui: &mut Ui, username: &str) {
        ui.vertical(|ui| {
            ui.heading(username);
            ui.separator();
            ui.spinner();
        });
    }

    fn make_buffer_image_view(&self, ui: &mut Ui, image_uri: &String, img_data: &Option<Arc<[u8]>>, full_view_uri: Option<&String>) {
        let mut s = DefaultHasher::new();
        image_uri.hash(&mut s);
        let hash = s.finish();
        let image_id = format!("bytes://{}.jpg", hash);

        if  let Some(data) = img_data {
            let img_view = ui.image(ImageSource::Bytes { 
                uri: Cow::from(image_id), 
                bytes: Bytes::Shared(data.clone())
            });
            let sensing_img = img_view.interact(egui::Sense::click());
    
            if sensing_img.clicked() {
                if let Some(uri) = full_view_uri {
                    dbg!(uri);
                    self.post_ui_message(RedskyUiMsg::ShowBigImageView { img_uri: uri.clone() });
                }
            }
        } else {
            ui.spinner();
        }


    }

    fn make_post_view(&self, ui: &mut Ui, _username: &str, posts: &Vec<Post>) {
        ui.vertical_centered_justified(|ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui|  {
                        for post in posts {
                            let post_block = ui.vertical(|ui|  {
                                ui.horizontal(|ui| {
                                    ui.set_min_height(48f32);
                                    if self.image_cache.contains_key(&post.avatar_img) {
                                        self.make_buffer_image_view(ui, &post.avatar_img, 
                                            self.image_cache.get(&post.avatar_img).unwrap(),
                                             Some(&post.avatar_img));                                    
                                            
                                    }

                                    ui.vertical(|ui| {
                                        if ui.link(RichText::new(&post.display_name).strong()).clicked() {
                                            self.post_ui_message(RedskyUiMsg::PrepareUserView { username: post.author.clone() });
                                            self.post_message(BskyActorMsg::GetUserPosts { username: post.author.clone() });
                                        };
                                        ui.label(&post.author);
                                        ui.label(RichText::new(&post.date).small())
                                    });

                                });
                                ui.style_mut().spacing.item_spacing = vec2(16.0, 16.0);
                                ui.label(&post.content);
                                if !&post.embeds.is_empty() {
                                    if ui.horizontal_wrapped(|ui|{
                                        ui.set_min_height(200f32);
                                        for embed in &post.embeds {
                                            if self.image_cache.contains_key(&embed.thumbnail_url) {
                                                self.make_buffer_image_view(ui, &embed.thumbnail_url,
                                                    self.image_cache.get(&embed.thumbnail_url).unwrap(),
                                                     Some(&embed.url));
                                            }
                                        }
                                    }).response.clicked() {
                                    };
                                }
                                ui.horizontal(|ui| {
                                    ui.button(format!("{} x ❤", &post.like_count));
                                    ui.button(format!("{} x 🔃", &post.repost_count));                                
                                    ui.button("…");
                                });
                                ui.separator();
                            });

                            if post_block.response.interact(Sense::click()).clicked() {
                                self.post_ui_message(RedskyUiMsg::PrepareThreadView { 
                                    thread_ref: StrongRef { 
                                    uri: post.uri.clone(), 
                                    cid : post.cid.clone()
                                 } });
                            }

                        }
                    });



    
            });
        });
    }

    fn make_user_timelines_views(&self, ctx: &egui::Context) {
        for (username, posts) in &self.user_posts {
            if username == &self.login {
                continue;
            }
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(username),
                egui::ViewportBuilder::default()
                    .with_title(format!("Posts of {}", username.clone()))
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

    
    fn make_open_thread_views(&self, ctx: &egui::Context) {
        for (repost_ref, posts) in &self.post_replies_cache {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(repost_ref.uri.clone()),
                egui::ViewportBuilder::default()
                    .with_title(format!("Thread {}", &repost_ref.uri))
                    .with_inner_size([400.0, 600.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        match posts {
                            Some(posts) => {

                                self.make_post_view(ui, "Thread", posts);
                            }
                            None => {
                                self.make_placeholder_post_view(ui, "Loading thread");
                            }
                        }
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.post_ui_message(RedskyUiMsg::CloseThreadView { thread_ref: repost_ref.clone() });
                    }
                });
        }
    }

    fn make_image_viewports(&self, ctx: &egui::Context) {
        for img in &self.opened_image_views  {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(img),
                egui::ViewportBuilder::default()
                    .with_title(format!("Viewing {}", img))
                    .with_inner_size([800.0, 600.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::both().show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                if self.image_cache.contains_key(img) {
                                    self.make_buffer_image_view(ui, img, self.image_cache.get(img).unwrap(), None);
                                } else {
                                    self.post_ui_message(RedskyUiMsg::PrepareImageView { img_uri: img.clone() });
                                }
                            });
                        });
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.post_ui_message(RedskyUiMsg::CloseBigImageView { img_uri: img.clone() });
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
        self.make_image_viewports(ctx);
        self.make_open_thread_views(ctx);
        
        if self.is_post_window_open {
            self.make_new_post_view(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui | {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New post...").clicked() {
                            self.is_post_window_open = true;
                        }
                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });
    
                });

                if self.main_view_state != MainViewState::Login {
                    ui.vertical(|ui| {
                        ui.horizontal_top(|ui|{
                            ui.selectable_value(&mut self.main_view_state, 
                                MainViewState::OwnPostFeed, RichText::new("Profile").heading());
                            ui.selectable_value(&mut self.main_view_state,
                                MainViewState::TimelineFeed, RichText::new("Timeline feed").heading());
                        });
                        ui.separator();
                    });
                }
                match self.main_view_state {
                    MainViewState::Login => {
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered_justified(|ui| {
                                ui.heading("Welcome to Redsky");
                                ui.separator();
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

