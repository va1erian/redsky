use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use atrium_api::types::string::Cid;
use egui::load::Bytes;
use egui::{vec2, Align, ImageSource, Layout, Sense, UiBuilder};
use egui::{RichText, Ui};
use egui_extras::{Size, StripBuilder};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct StrongRef {
    pub uri: String,
    pub cid: Cid
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserProfile {
    pub handle: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_uri: String,
    pub follower_count: i64,
    pub follow_count: i64,
    pub post_count: i64
}

#[derive(Clone)]
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
}

pub enum FeedItem {
    Full(Post),
    Dehydrated { uri: String },
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
    PostSucceeed(),
    PrepareUserView{username: String},
    PrepareThreadView{thread_ref: StrongRef},
    CloseThreadView{thread_ref: StrongRef},
    NotifyImageLoaded{url: String, data: Arc<[u8]>},
    NotifyLikesLoaded {post_uri: StrongRef, likers: Vec<UserProfile> },
    NotifyPostAndRepliesLoaded {post: Post, replies : Vec<Post>},
    ShowUserProfile{profile: UserProfile},
    RefreshTimelineMsg{posts: Vec<Post>, cursor: Option<String>, append: bool},
    ShowUserPostsMsg{username: String, posts: Vec<Post>, cursor: Option<String>, append: bool},
    DropUserPostsMsg{username: String},
    PrepareImageView {img_uri: String},
    ShowBigImageView {img_uri: String},
    CloseBigImageView {img_uri: String}, 
    ShowErrorMsg{error: String},
    DownloadProgress { id: u64, processed_posts: usize, total_posts: Option<usize>, downloaded_images: usize, total_images: Option<usize>, status: DownloadStatus },
    DownloadFinished { id: u64, errors: Vec<String> },
    StartDownloadJob { username: String, path: String },
    ShowSearchResults { results: Vec<UserProfile> }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BskyActorMsg {
    Login {login: String, pass: String},
    Post {msg_body: String},
    GetTimeline { cursor: Option<String> },
    //GetPostLikers {post_ref: StrongRef},
    GetPostAndReplies {post_ref: StrongRef},
    GetUserProfile{username: String},
    GetUserPosts {username: String, cursor: Option<String>},
    SearchActors { query: String },
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
    timeline: Vec<FeedItem>,
    user_posts: HashMap<String, Option<Vec<FeedItem>>>,
    timeline_cursor: Option<String>,
    user_cursors: HashMap<String, Option<String>>,
    post_cache: HashMap<String, Post>,
    post_cache_order: VecDeque<String>,
    scroll_to_top: bool,
    
    user_infos_cache: HashMap<String, UserProfile>,
    image_cache: HashMap<String, Option<Arc<[u8]>>>,
    post_likers_cache: HashMap<StrongRef, Vec<UserProfile>>,
    post_replies_cache: HashMap<StrongRef, Option<Vec<FeedItem>>>,
    opened_image_views: HashSet<String>,
    download_tasks: HashMap<u64, DownloadTask>,
    next_download_id: u64,
    is_search_window_open: bool,
    search_query: String,
    search_results: Vec<UserProfile>
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
            timeline_cursor: None,
            user_cursors: HashMap::new(),
            post_cache: HashMap::new(),
            post_cache_order: VecDeque::new(),
            scroll_to_top: false,
            user_infos_cache: HashMap::new(),
            image_cache: HashMap::new(),
            post_likers_cache: HashMap::new(),
            post_replies_cache: HashMap::new(),
            opened_image_views: HashSet::new(),
            download_tasks: HashMap::new(),
            next_download_id: 0,
            is_search_window_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
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
        if img_url.is_empty() {
            return;
        }
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

    fn process_message(&mut self, msg: RedskyUiMsg) -> () {
        
        match msg {
            RedskyUiMsg::PostSucceeed () => {
                self.post_message(BskyActorMsg::GetTimeline { cursor: None });
            }
            RedskyUiMsg::RefreshTimelineMsg { posts, cursor, append } => {
                self.request_post_images(&posts);
                let new_items: Vec<FeedItem> = posts.into_iter().map(FeedItem::Full).collect();
                if append {
                    self.timeline.extend(new_items);
                } else {
                    self.timeline = new_items;
                }
                self.timeline_cursor = cursor;
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
            RedskyUiMsg::ShowUserPostsMsg { username, posts, cursor, append }  => {
                self.request_post_images(&posts);
                let new_items: Vec<FeedItem> = posts.into_iter().map(FeedItem::Full).collect();
                if append {
                    if let Some(Some(existing_posts)) = self.user_posts.get_mut(&username) {
                        existing_posts.extend(new_items);
                    }
                } else {
                    self.user_posts.insert(username.clone(), Some(new_items));
                }
                self.user_cursors.insert(username, cursor);
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
            RedskyUiMsg::NotifyPostAndRepliesLoaded { post, replies } => {
                let strong_ref = StrongRef { uri: post.uri.clone(), cid: post.cid.clone()};
                self.request_post_images(&replies);
                self.request_post_images(&vec![post.clone()]);

                let mut items: Vec<FeedItem> = vec![FeedItem::Full(post)];
                items.extend(replies.into_iter().map(FeedItem::Full));

                self.post_replies_cache.insert(strong_ref, Some(items));
            }
            RedskyUiMsg::LogInSucceededMsg() => {
                self.is_logged_in = true;
                self.main_view_state = MainViewState::OwnPostFeed;
                self.post_message(BskyActorMsg::GetUserPosts{username: self.login.clone(), cursor: None});
                self.post_message(BskyActorMsg::GetUserProfile { username: self.login.clone() });
                self.post_message(BskyActorMsg::GetTimeline { cursor: None });
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
            RedskyUiMsg::ShowSearchResults { results } => {
                for profile in &results {
                    self.request_image(&profile.avatar_uri);
                }
                self.search_results = results;
            }
        }
    }

    fn make_maybe_user_profile_view(&self, ui: &mut Ui, username: &str, maybe_profile: Option<&UserProfile>) {
        match maybe_profile {
            Some(profile) => {
                ui.horizontal(|ui| {
                    ui.set_max_height(120f32);
                    match self.image_cache.get(&profile.avatar_uri) {
                        Some(Some(img)) => {
                            let mut s = DefaultHasher::new();
                            profile.avatar_uri.hash(&mut s);
                            let hash = s.finish();
                            let image_id = format!("bytes://{}.jpg", hash);

                                ui.vertical(|ui| {
                                    ui.set_max_width(120f32);
                                    ui.set_max_height(120f32);
                                    ui.image(ImageSource::Bytes { 
                                        uri: Cow::from(image_id), 
                                        bytes: Bytes::Shared(img.clone())
                                    });
                                });
                        }
                        Some(None) => {
                            ui.vertical(|ui| {
                                ui.set_max_width(120f32);
                                ui.set_max_height(120f32);
                                ui.spinner();
                            });
                        }
                        None => {
                            self.post_message(BskyActorMsg::LoadImage { url: profile.avatar_uri.clone() });
                        }
                    }
                    ui.vertical(|ui|{
                        ui.set_max_height(120f32);
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

    fn make_maybe_user_post_view(&mut self, ui: &mut Ui, username: &str, posts: &mut Option<Vec<FeedItem>>) {
        StripBuilder::new(ui)
        .size(Size::exact(120.0))
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

    fn make_placeholder_post_view(&mut self, ui: &mut Ui, username: &str) {
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
            ui.set_min_height(57.6f32);
            if self.image_cache.contains_key(&post.avatar_img) {
                ui.vertical(|ui| {
                    ui.set_max_width(57.6f32);
                    self.make_buffer_image_view(ui, &post.avatar_img,
                        self.image_cache.get(&post.avatar_img).unwrap(),
                         Some(&post.avatar_img));
                });
            }

            ui.vertical(|ui| {
                if ui.link(RichText::new(&post.display_name).strong()).clicked() {
                    self.post_ui_message(RedskyUiMsg::PrepareUserView { username: post.author.clone() });
                    self.post_message(BskyActorMsg::GetUserPosts { username: post.author.clone(), cursor: None });
                };
                ui.label(&post.author);
                ui.label(RichText::new(&post.date).small())
            });

        });
        ui.style_mut().spacing.item_spacing = vec2(16.0, 16.0);
        ui.label(&post.content);
    }


    fn make_post_view(&mut self, ui: &mut Ui, username: &str, posts: &mut Vec<FeedItem>) {
        let mut scroll_top_reset = false;
        let mut scroll_offset_y = 0.0;
        let mut content_size_y = 0.0;

        ui.vertical_centered_justified(|ui| {
            let scroll_area = egui::ScrollArea::vertical();
            let scroll_output = scroll_area.show(ui, |ui| {
                ui.vertical(|ui| {
                    for (idx, item) in posts.iter_mut().enumerate() {
                        match item {
                            FeedItem::Full(post) => {
                                let post_block = ui.vertical(|ui| {
                                    if idx == 0 && self.scroll_to_top {
                                        ui.scroll_to_rect(ui.max_rect(), Some(egui::Align::TOP));
                                        scroll_top_reset = true;
                                    }
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
                                        ui.horizontal_wrapped(|ui| {
                                            ui.set_min_height(200f32);
                                            for embed in &post.embeds {
                                                if self.image_cache.contains_key(&embed.thumbnail_url) {
                                                    self.make_buffer_image_view(
                                                        ui,
                                                        &embed.thumbnail_url,
                                                        self.image_cache.get(&embed.thumbnail_url).unwrap(),
                                                        Some(&embed.url),
                                                    );
                                                }
                                            }
                                        });
                                    }
                                    ui.horizontal(|ui| {
                                        let _ = ui.button(format!("{} x ❤", &post.like_count));
                                        let _ = ui.button(format!("{} x 🔃", &post.repost_count));
                                        let _ = ui.button("…");
                                    });
                                    ui.separator();
                                });

                                if post_block.response.interact(Sense::click()).clicked() {
                                    self.post_ui_message(RedskyUiMsg::PrepareThreadView {
                                        thread_ref: StrongRef {
                                            uri: post.uri.clone(),
                                            cid: post.cid.clone(),
                                        },
                                    });
                                }
                            }
                            FeedItem::Dehydrated { uri } => {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(50.0);
                                    ui.spinner();
                                    ui.add_space(50.0);
                                });
                            }
                        }

                        // Rehydration check
                        let mut rehydrate_uri = None;
                        if let FeedItem::Dehydrated { uri } = item {
                            if ui.is_rect_visible(ui.available_rect_before_wrap()) {
                                rehydrate_uri = Some(uri.clone());
                            }
                        }
                        if let Some(uri) = rehydrate_uri {
                            if let Some(post) = self.post_cache.remove(&uri) {
                                *item = FeedItem::Full(post);
                                self.post_cache_order.retain(|u| u != &uri);
                            }
                        }
                    }
                });
            });
            scroll_offset_y = scroll_output.state.offset.y;
            content_size_y = scroll_output.content_size.y;

            // Infinite Scroll Check
            if scroll_offset_y > content_size_y * 0.8 && content_size_y > 0.0 {
                if username == "Your timeline" {
                    if let Some(cursor) = self.timeline_cursor.clone() {
                        self.post_message(BskyActorMsg::GetTimeline { cursor: Some(cursor) });
                        self.timeline_cursor = None; // Avoid duplicate requests
                    }
                } else if username != "Thread" {
                    if let Some(cursor) = self.user_cursors.get(username).cloned().flatten() {
                        self.post_message(BskyActorMsg::GetUserPosts { username: username.to_string(), cursor: Some(cursor) });
                        self.user_cursors.insert(username.to_string(), None); // Avoid duplicate requests
                    }
                }
            }
        });

        if scroll_top_reset {
            self.scroll_to_top = false;
        }

        // Dehydration logic
        let visible_idx = (scroll_offset_y / 200.0) as i32;
        for (idx, item) in posts.iter_mut().enumerate() {
            let mut should_dehydrate = false;
            if let FeedItem::Full(_) = item {
                if (idx as i32 - visible_idx).abs() > 50 {
                    should_dehydrate = true;
                }
            }

            if should_dehydrate {
                if let FeedItem::Full(post) = std::mem::replace(item, FeedItem::Dehydrated { uri: String::new() }) {
                    let uri = post.uri.clone();
                    if !uri.is_empty() {
                        self.post_cache.insert(uri.clone(), post);
                        self.post_cache_order.push_back(uri.clone());

                        // LRU Eviction
                        if self.post_cache.len() > 200 {
                            if let Some(oldest_uri) = self.post_cache_order.pop_front() {
                                self.post_cache.remove(&oldest_uri);
                            }
                        }
                        *item = FeedItem::Dehydrated { uri };
                    }
                }
            }
        }
    }

    fn make_user_timelines_views(&mut self, ctx: &egui::Context) {
        let mut to_drop = Vec::new();
        let mut to_download = Vec::new();

        let usernames: Vec<String> = self.user_posts.keys().cloned().collect();
        for username in usernames {
            if username == self.login {
                continue;
            }

            let mut posts = self.user_posts.get_mut(&username).unwrap().take();

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(&username),
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
                        self.make_maybe_user_post_view(ui, &username, &mut posts);
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        to_drop.push(username.clone());
                    }
                });
            self.user_posts.insert(username, posts);
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

    
    fn make_open_thread_views(&mut self, ctx: &egui::Context) {
        let keys: Vec<StrongRef> = self.post_replies_cache.keys().cloned().collect();
        for repost_ref in keys {
            let mut posts_opt = self.post_replies_cache.get_mut(&repost_ref).unwrap().take();

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(repost_ref.uri.clone()),
                egui::ViewportBuilder::default()
                    .with_title(format!("Thread {}", &repost_ref.uri))
                    .with_inner_size([400.0, 600.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        match &mut posts_opt {
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
            self.post_replies_cache.insert(repost_ref, posts_opt);
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

    fn make_search_window(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("__search_actors"),
            egui::ViewportBuilder::default()
                .with_title("Search Accounts")
                .with_inner_size([400.0, 500.0]),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Search Accounts");
                        if ui.text_edit_singleline(&mut self.search_query).changed() {
                            self.post_message(BskyActorMsg::SearchActors { query: self.search_query.clone() });
                        }
                        ui.separator();

                        let mut clicked_profile = None;
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for profile in &self.search_results {
                                let (rect, response) = ui.allocate_at_least(vec2(ui.available_width(), 57.6), Sense::click());
                                if response.hovered() {
                                    ui.painter().rect_filled(rect, 4.0, egui::Color32::from_gray(64));
                                }
                                if response.clicked() {
                                    clicked_profile = Some(profile.clone());
                                }

                                let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
                                child_ui.horizontal(|ui| {
                                    ui.add_space(4.0);
                                    ui.vertical(|ui| {
                                        ui.set_max_width(57.6f32);
                                        self.make_buffer_image_view(ui, &profile.avatar_uri, self.image_cache.get(&profile.avatar_uri).unwrap_or(&None), None);
                                    });
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(&profile.display_name).strong());
                                        ui.small(&profile.handle);
                                    });
                                });
                            }
                        });
                        if let Some(profile) = clicked_profile {
                            self.post_ui_message(RedskyUiMsg::PrepareUserView { username: profile.handle.clone() });
                            self.post_message(BskyActorMsg::GetUserPosts { username: profile.handle.clone() });
                            self.is_search_window_open = false;
                        }
                    });
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.is_search_window_open = false;
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
        
        if self.is_post_window_open {
            self.make_new_post_view(ctx);
        }

        if self.is_search_window_open {
            self.make_search_window(ctx);
        }

        if self.main_view_state != MainViewState::Login {
            let mut top_clicked = false;
            egui::Area::new(egui::Id::new("top_button"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
                .show(ctx, |ui| {
                    if ui.button(RichText::new("Top").heading()).clicked() {
                        top_clicked = true;
                    }
                });
            if top_clicked {
                self.scroll_to_top = true;
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui | {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New post...").clicked() {
                            self.is_post_window_open = true;
                        }
                        if ui.button("Search accounts...").clicked() {
                            self.is_search_window_open = true;
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
                                let mut timeline = std::mem::take(&mut self.timeline);
                                self.make_post_view(ui, "Your timeline", &mut timeline);
                                self.timeline = timeline;
                            });
                        });
                    }
                    MainViewState::OwnPostFeed => {
                        let login = self.login.clone();
                        let mut maybe_post = self.user_posts.get_mut(&login).and_then(|p| p.take());
                        self.make_maybe_user_post_view(ui, &login, &mut maybe_post);
                        self.user_posts.insert(login, maybe_post);
                    }
                }
            })

        });
    }
}

