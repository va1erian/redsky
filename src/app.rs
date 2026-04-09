use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use atrium_api::types::string::Cid;
use egui::load::Bytes;
use egui::{vec2, ImageSource, Sense};
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
pub struct DownloadTask {
    pub id: u64,
    pub username: String,
    pub path: String,
    pub processed_posts: usize,
    pub total_posts: Option<usize>,
    pub downloaded_images: usize,
    pub total_images: Option<usize>,
    pub status: DownloadStatus,
    pub errors: Vec<String>,
}

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
    pub embeds: Vec<PostImage>,
    pub quoted_post: Option<Box<Post>>,
    pub is_reply: bool,
    pub viewer_like: Option<String>,
    pub viewer_repost: Option<String>,
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
    pub thumbnail_url: String,
    pub url: String,
    pub alt: String
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DownloadStatus {
    Scanning,
    Downloading,
    Finished,
    Cancelled,
}

pub enum RedskyUiMsg {
    LogInSucceededMsg(),
    ActionSucceeded(),
    PrepareUserView{username: String},
    PrepareThreadView{thread_ref: StrongRef},
    CloseThreadView{thread_ref: StrongRef},
    NotifyImageLoaded{url: String, data: Arc<[u8]>},
    NotifyLikesLoaded {post_uri: StrongRef, likers: Vec<UserProfile> },
    NotifyRepostersLoaded {post_uri: StrongRef, reposters: Vec<UserProfile> },
    CloseLikesView { post_uri: StrongRef },
    CloseRepostersView { post_uri: StrongRef },
    PrepareLikeAction { post_uri: String, post_cid: Cid, unlike: bool },
    PrepareRepostAction { post_uri: String, post_cid: Cid, unrepost: bool },
    NotifyLikeActionSucceeded { post_uri: String, like_uri: String },
    NotifyRepostActionSucceeded { post_uri: String, repost_uri: String },
    NotifyPostAndRepliesLoaded {post: Post, replies : Vec<Post>},
    ShowUserProfile{profile: UserProfile},
    RefreshTimelineMsg{posts: Vec<Post>},
    ShowUserPostsMsg{username: String, posts: Vec<Post>},
    DropUserPostsMsg{username: String},
    PrepareImageView {img_uri: String},
    ShowBigImageView {img_uri: String},
    CloseBigImageView {img_uri: String}, 
    ShowErrorMsg{error: String},
    DownloadProgress { id: u64, processed_posts: usize, total_posts: Option<usize>, downloaded_images: usize, total_images: Option<usize>, status: DownloadStatus },
    DownloadFinished { id: u64, errors: Vec<String> },
    StartDownloadJob { username: String, path: String }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BskyActorMsg {
    Login {login: String, pass: String},
    Post {msg_body: String},
    GetTimeline(),
    Like {post_ref: StrongRef},
    Unlike {post_uri: String, like_record_uri: String},
    Repost {post_ref: StrongRef},
    Unrepost {post_uri: String, repost_record_uri: String},
    GetPostLikers {post_ref: StrongRef},
    GetPostRepostedBy {post_ref: StrongRef},
    GetPostAndReplies {post_ref: StrongRef},
    GetUserProfile{username: String},
    GetUserPosts {username: String},
    LoadImage{url: String},
    StartImageDownload { id: u64, username: String, path: String },
    CancelImageDownload { id: u64 },
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
    post_reposters_cache: HashMap<StrongRef, Vec<UserProfile>>,
    post_replies_cache: HashMap<StrongRef, Option<Vec<Post>>>,
    opened_image_views: HashSet<String>,
    download_tasks: HashMap<u64, DownloadTask>,
    next_download_id: u64,
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
            post_reposters_cache: HashMap::new(),
            post_replies_cache: HashMap::new(),
            opened_image_views: HashSet::new(),
            download_tasks: HashMap::new(),
            next_download_id: 0,
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

    fn request_image(&mut self, img_url: &str) {
        if let None = self.image_cache.get(img_url) {
            println!("requesting image {}", img_url);
            self.image_cache.insert(img_url.to_string(), None);
            self.post_message(BskyActorMsg::LoadImage { url: img_url.to_string()});
        }
    }

    fn request_post_images(&mut self, posts: &Vec<Post>) {
        for post in posts {
            self.request_image(&post.avatar_img );

            if let Some(quoted_post) = &post.quoted_post {
                self.request_image(&quoted_post.avatar_img);
            }

            for embed in &post.embeds {
                self.request_image(&embed.thumbnail_url );
            }

        }
    }

    fn update_post_optimistically<F>(&mut self, post_uri: &str, update_fn: F)
    where F: Fn(&mut Post) {
        // Update timeline
        for post in &mut self.timeline {
            if post.uri == post_uri {
                update_fn(post);
            }
            if let Some(quoted) = &mut post.quoted_post {
                if quoted.uri == post_uri {
                    update_fn(quoted);
                }
            }
        }
        // Update user posts
        for posts in self.user_posts.values_mut() {
            if let Some(posts) = posts {
                for post in posts {
                    if post.uri == post_uri {
                        update_fn(post);
                    }
                    if let Some(quoted) = &mut post.quoted_post {
                        if quoted.uri == post_uri {
                            update_fn(quoted);
                        }
                    }
                }
            }
        }
        // Update replies cache
        for posts in self.post_replies_cache.values_mut() {
            if let Some(posts) = posts {
                for post in posts {
                    if post.uri == post_uri {
                        update_fn(post);
                    }
                    if let Some(quoted) = &mut post.quoted_post {
                        if quoted.uri == post_uri {
                            update_fn(quoted);
                        }
                    }
                }
            }
        }
    }

    fn process_message(&mut self, msg: RedskyUiMsg) -> () {
        
        match msg {
            RedskyUiMsg::ActionSucceeded () => {
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
            RedskyUiMsg::NotifyRepostersLoaded { post_uri, reposters } => {
                self.post_reposters_cache.insert(post_uri, reposters);
            }
            RedskyUiMsg::CloseLikesView { post_uri } => {
                self.post_likers_cache.remove(&post_uri);
            }
            RedskyUiMsg::CloseRepostersView { post_uri } => {
                self.post_reposters_cache.remove(&post_uri);
            }
            RedskyUiMsg::PrepareLikeAction { post_uri, post_cid, unlike } => {
                if unlike {
                    let mut like_record_uri = String::new();
                    self.update_post_optimistically(&post_uri, |post| {
                        if let Some(uri) = &post.viewer_like {
                            like_record_uri = uri.clone();
                        }
                        post.viewer_like = None;
                        post.like_count = (post.like_count - 1).max(0);
                    });
                    if !like_record_uri.is_empty() {
                        self.post_message(BskyActorMsg::Unlike { post_uri, like_record_uri });
                    }
                } else {
                    self.update_post_optimistically(&post_uri, |post| {
                        post.viewer_like = Some("pending".to_string());
                        post.like_count += 1;
                    });
                    self.post_message(BskyActorMsg::Like { post_ref: StrongRef { uri: post_uri, cid: post_cid } });
                }
            }
            RedskyUiMsg::PrepareRepostAction { post_uri, post_cid, unrepost } => {
                if unrepost {
                    let mut repost_record_uri = String::new();
                    self.update_post_optimistically(&post_uri, |post| {
                        if let Some(uri) = &post.viewer_repost {
                            repost_record_uri = uri.clone();
                        }
                        post.viewer_repost = None;
                        post.repost_count = (post.repost_count - 1).max(0);
                    });
                    if !repost_record_uri.is_empty() {
                        self.post_message(BskyActorMsg::Unrepost { post_uri, repost_record_uri });
                    }
                } else {
                    self.update_post_optimistically(&post_uri, |post| {
                        post.viewer_repost = Some("pending".to_string());
                        post.repost_count += 1;
                    });
                    self.post_message(BskyActorMsg::Repost { post_ref: StrongRef { uri: post_uri, cid: post_cid } });
                }
            }
            RedskyUiMsg::NotifyLikeActionSucceeded { post_uri, like_uri } => {
                self.update_post_optimistically(&post_uri, |post| {
                    post.viewer_like = Some(like_uri.clone());
                });
            }
            RedskyUiMsg::NotifyRepostActionSucceeded { post_uri, repost_uri } => {
                self.update_post_optimistically(&post_uri, |post| {
                    post.viewer_repost = Some(repost_uri.clone());
                });
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
            RedskyUiMsg::DownloadProgress { id, processed_posts, total_posts, downloaded_images, total_images, status } => {
                if let Some(task) = self.download_tasks.get_mut(&id) {
                    task.processed_posts = processed_posts;
                    task.total_posts = total_posts;
                    task.downloaded_images = downloaded_images;
                    task.total_images = total_images;
                    task.status = status;
                }
            }
            RedskyUiMsg::DownloadFinished { id, errors } => {
                if let Some(task) = self.download_tasks.get_mut(&id) {
                    task.status = DownloadStatus::Finished;
                    task.errors = errors;
                }
            }
            RedskyUiMsg::StartDownloadJob { username, path } => {
                let id = self.next_download_id;
                self.next_download_id += 1;
                self.download_tasks.insert(id, DownloadTask {
                    id,
                    username: username.clone(),
                    path: path.clone(),
                    processed_posts: 0,
                    total_posts: None,
                    downloaded_images: 0,
                    total_images: None,
                    status: DownloadStatus::Scanning,
                    errors: Vec::new(),
                });
                self.post_message(BskyActorMsg::StartImageDownload { id, username, path });
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

    fn make_post_inner_view(&self, ui: &mut Ui, post: &Post) {
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
    }


    fn make_post_view(&self, ui: &mut Ui, _username: &str, posts: &Vec<Post>) {
        ui.vertical_centered_justified(|ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui|  {
                        for post in posts {
                            let post_block = ui.vertical(|ui|  {
                                self.make_post_inner_view(ui, post);

                                if let Some(quoted_post) = &post.quoted_post {
                                    egui::Frame::new()
                                        .inner_margin(8)
                                        .outer_margin(8)
                                        .corner_radius(8)
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                                        .show(ui, |ui| {
                                            self.make_post_inner_view(ui, &quoted_post);
                                        });       
                               }

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
                                    let like_text = if post.viewer_like.is_some() {
                                        RichText::new(format!("{} x ❤", &post.like_count)).color(egui::Color32::RED)
                                    } else {
                                        RichText::new(format!("{} x ❤", &post.like_count))
                                    };
                                    let like_btn = ui.button(like_text);
                                    if like_btn.clicked() {
                                        let post_uri = post.uri.clone();
                                        let post_cid = post.cid.clone();
                                        self.post_ui_message(RedskyUiMsg::PrepareLikeAction {
                                            post_uri,
                                            post_cid,
                                            unlike: post.viewer_like.is_some()
                                        });
                                    }
                                    like_btn.context_menu(|ui| {
                                        if ui.button("Show Likers").clicked() {
                                            self.post_message(BskyActorMsg::GetPostLikers {
                                                post_ref: StrongRef {
                                                    uri: post.uri.clone(),
                                                    cid: post.cid.clone()
                                                }
                                            });
                                            ui.close_menu();
                                        }
                                    });

                                    let repost_text = if post.viewer_repost.is_some() {
                                        RichText::new(format!("{} x 🔃", &post.repost_count)).color(egui::Color32::GREEN)
                                    } else {
                                        RichText::new(format!("{} x 🔃", &post.repost_count))
                                    };
                                    let repost_btn = ui.button(repost_text);
                                    if repost_btn.clicked() {
                                        let post_uri = post.uri.clone();
                                        let post_cid = post.cid.clone();
                                        self.post_ui_message(RedskyUiMsg::PrepareRepostAction {
                                            post_uri,
                                            post_cid,
                                            unrepost: post.viewer_repost.is_some()
                                        });
                                    }
                                    repost_btn.context_menu(|ui| {
                                        if ui.button("Show Reposters").clicked() {
                                            self.post_message(BskyActorMsg::GetPostRepostedBy {
                                                post_ref: StrongRef {
                                                    uri: post.uri.clone(),
                                                    cid: post.cid.clone()
                                                }
                                            });
                                            ui.close_menu();
                                        }
                                    });

                                    let _ = ui.button("…");
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

    fn make_user_timelines_views(&mut self, ctx: &egui::Context) {
        let mut to_drop = Vec::new();
        let mut to_download = Vec::new();

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
                    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                        egui::menu::bar(ui, |ui| {
                            ui.menu_button("Actions", |ui| {
                                if ui.button("Download All Images").clicked() {
                                    to_download.push(username.clone());
                                    ui.close_menu();
                                }
                            });
                        });
                    });
                    egui::CentralPanel::default().show(ctx, |ui| {
                        self.make_maybe_user_post_view(ui, username, posts);
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        to_drop.push(username.clone());
                    }
                });
        }

        for username in to_drop {
            self.ui_tx.send(RedskyUiMsg::DropUserPostsMsg { username }).unwrap();
        }

        for username in to_download {
            let ui_tx = self.ui_tx.clone();
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let path_str = path.display().to_string();
                    let _ = ui_tx.send(RedskyUiMsg::StartDownloadJob { username, path: path_str });
                    ctx.request_repaint();
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

    fn make_user_list_viewports(&self, ctx: &egui::Context) {
        for (post_ref, likers) in &self.post_likers_cache {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(format!("likers_{}", post_ref.uri)),
                egui::ViewportBuilder::default()
                    .with_title(format!("Likers of post"))
                    .with_inner_size([300.0, 400.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for user in likers {
                                self.make_maybe_user_profile_view(ui, &user.handle, Some(user));
                                ui.separator();
                            }
                        });
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.post_ui_message(RedskyUiMsg::CloseLikesView { post_uri: post_ref.clone() });
                    }
                }
            );
        }
        for (post_ref, reposters) in &self.post_reposters_cache {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(format!("reposters_{}", post_ref.uri)),
                egui::ViewportBuilder::default()
                    .with_title(format!("Reposters of post"))
                    .with_inner_size([300.0, 400.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for user in reposters {
                                self.make_maybe_user_profile_view(ui, &user.handle, Some(user));
                                ui.separator();
                            }
                        });
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.post_ui_message(RedskyUiMsg::CloseRepostersView { post_uri: post_ref.clone() });
                    }
                }
            );
        }
    }

    fn make_download_progress_view(&mut self, ctx: &egui::Context) {
        let mut to_remove = Vec::new();
        let mut to_cancel = Vec::new();

        for (id, task) in &self.download_tasks {
            let mut open = true;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(format!("download_{}", id)),
                egui::ViewportBuilder::default()
                    .with_title(format!("Downloading images for {}", task.username))
                    .with_inner_size([400.0, 300.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.heading(format!("Target: {}", task.path));
                            ui.separator();

                            match task.status {
                                DownloadStatus::Scanning => {
                                    ui.label(format!("Scanning posts... ({})", task.processed_posts));
                                    ui.add(egui::ProgressBar::new(0.0).animate(true));
                                }
                                DownloadStatus::Downloading => {
                                    let progress = if let Some(total) = task.total_images {
                                        if total > 0 {
                                            task.downloaded_images as f32 / total as f32
                                        } else {
                                            1.0
                                        }
                                    } else {
                                        0.0
                                    };
                                    ui.label(format!("Downloading images... ({}/{})",
                                        task.downloaded_images,
                                        task.total_images.unwrap_or(0)));
                                    ui.add(egui::ProgressBar::new(progress).show_percentage());
                                }
                                DownloadStatus::Finished => {
                                    ui.label("Finished!");
                                    ui.add(egui::ProgressBar::new(1.0));
                                }
                                DownloadStatus::Cancelled => {
                                    ui.label("Cancelled");
                                }
                            }

                            if !task.errors.is_empty() {
                                ui.separator();
                                ui.label("Errors:");
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    for error in &task.errors {
                                        ui.colored_label(egui::Color32::RED, error);
                                    }
                                });
                            }

                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                                if task.status == DownloadStatus::Finished || task.status == DownloadStatus::Cancelled {
                                    if ui.button("Close").clicked() {
                                        open = false;
                                    }
                                } else {
                                    if ui.button("Cancel").clicked() {
                                        to_cancel.push(*id);
                                    }
                                }
                            });
                        });
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        open = false;
                    }
                }
            );
            if !open {
                to_remove.push(*id);
            }
        }

        for id in to_cancel {
            if let Some(task) = self.download_tasks.get_mut(&id) {
                task.status = DownloadStatus::Cancelled;
                self.post_message(BskyActorMsg::CancelImageDownload { id });
            }
        }

        for id in to_remove {
            self.download_tasks.remove(&id);
            self.post_message(BskyActorMsg::CancelImageDownload { id });
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
        

        while let Ok(msg) = self.rx.try_recv() {
            self.process_message(msg);
        }
        
        self.make_user_timelines_views(ctx);
        self.make_download_progress_view(ctx);
        self.make_image_viewports(ctx);
        self.make_open_thread_views(ctx);
        self.make_user_list_viewports(ctx);
        
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
                                MainViewState::TimelineFeed, RichText::new("Timeline feed").heading())
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

