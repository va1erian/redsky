use atrium_api::types::string::Cid;
use egui::{vec2, Align, Layout, Sense, UiBuilder};
use egui::{RichText, Ui};
use egui_extras::{Size, StripBuilder};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
include!("types.rs");
include!("screenshot_state.rs");

const KEYRING_SERVICE: &str = "redsky";
const KEYRING_USER: &str = "credentials";

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
    bookmarks: Vec<Post>,
    user_posts: HashMap<String, Option<Vec<FeedItem>>>,
    user_likes_posts: HashMap<String, Option<Vec<FeedItem>>>,
    user_view_states: HashMap<String, UserViewState>,
    media_image_sizes: HashMap<String, f32>,
    timeline_cursor: Option<String>,
    user_cursors: HashMap<String, Option<String>>,
    user_likes_cursors: HashMap<String, Option<String>>,
    post_cache: HashMap<String, Post>,
    post_cache_order: VecDeque<String>,
    scroll_to_top: bool,
    user_infos_cache: HashMap<String, UserProfile>,
    image_cache: HashMap<String, Option<egui::TextureHandle>>,
    post_likers_cache: HashMap<StrongRef, Vec<UserProfile>>,
    post_reposters_cache: HashMap<StrongRef, Vec<UserProfile>>,
    post_replies_cache: HashMap<StrongRef, Option<Vec<FeedItem>>>,
    opened_image_views: HashSet<String>,
    opened_raw_views: HashMap<String, String>, // uri -> raw_json
    download_tasks: HashMap<u64, DownloadTask>,
    next_download_id: u64,
    is_search_window_open: bool,
    search_query: String,
    search_results: Vec<UserProfile>,
    is_search_posts_window_open: bool,
    search_posts_query: String,
    search_posts_results: Option<Vec<FeedItem>>,
    search_posts_cursor: Option<String>,
    unread_notifications: i64,
    notifications: Vec<AppNotification>,
    remember_me: bool,
    pub settings: AppSettings,
    pub is_settings_window_open: bool,
    new_post_images: Vec<String>,
    reply_to: Option<(StrongRef, StrongRef)>,
    screenshot_requested: bool,
    #[allow(dead_code)]
    screenshot_output_path: Option<String>,
    frames_rendered: usize,
    screenshot_state: Option<ScreenshotState>,
}
impl RedskyApp {
    pub fn new(
        tx: Sender<BskyActorMsg>,
        ui_tx: Sender<RedskyUiMsg>,
        rx: Receiver<RedskyUiMsg>,
        is_screenshot_mode: bool,
        screenshot_output_path: Option<String>,
    ) -> Self {
        let mut login = String::new();
        let mut pass = String::new();
        let mut remember_me = false;

        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
            if let Ok(cred) = entry.get_password() {
                if let Some((l, p)) = cred.split_once(':') {
                    login = l.to_string();
                    pass = p.to_string();
                    remember_me = true;
                    if !is_screenshot_mode {
                        let _ = tx.send(BskyActorMsg::Login {
                            login: login.clone(),
                            pass: pass.clone(),
                        });
                    }
                }
            }
        }

        let mut timeline = Vec::new();
        let mut main_view_state = MainViewState::Login;
        let mut is_logged_in = false;
        #[allow(unused_mut, unused_variables)]
        let mut opened_image_views: HashSet<String> = HashSet::new();
        let mut screenshot_state = None;

        if is_screenshot_mode {
            is_logged_in = true;
            screenshot_state = Some(ScreenshotState::Timeline);
            main_view_state = MainViewState::TimelineFeed;
            timeline.push(FeedItem::Full(Post {
                uri: "at://mock-uri".to_string(),
                cid: "bafyreidfzuflltehrwqx5dzlqg3vzd2q6fudx75h7m3e7y22qpxg3ntv6m".parse().unwrap(),
                content: "Hello, world! This is a mock post for the screenshot test.".to_string(),
                author: "mockuser.bsky.social".to_string(),
                display_name: "Mock User".to_string(),
                avatar_img: "".to_string(),
                date: "2024-01-01T00:00:00Z".to_string(),
                like_count: 42,
                repost_count: 7,
                embeds: vec![],
                quoted_post: None,
                is_reply: false,
                viewer_like: None,
                viewer_repost: None,
                thread_root: None,
                raw_json: "{}".to_string(),
            }));
            timeline.push(FeedItem::Full(Post {
                uri: "at://mock-uri-2".to_string(),
                cid: "bafyreidfzuflltehrwqx5dzlqg3vzd2q6fudx75h7m3e7y22qpxg3ntv6m".parse().unwrap(),
                content: "Another mock post right here.".to_string(),
                author: "anotheruser.bsky.social".to_string(),
                display_name: "Another User".to_string(),
                avatar_img: "".to_string(),
                date: "2024-01-01T00:05:00Z".to_string(),
                like_count: 100,
                repost_count: 20,
                embeds: vec![],
                quoted_post: None,
                is_reply: false,
                viewer_like: None,
                viewer_repost: None,
                thread_root: None,
                raw_json: "{}".to_string(),
            }));
            let _ = tx.send(BskyActorMsg::GetUnreadCount());
        }

        Self {
            tx,
            ui_tx,
            rx,
            is_logged_in,
            is_post_window_open: is_screenshot_mode,
            main_view_state,
            login,
            pass,
            remember_me,
            msg: String::new(),
            timeline,
            bookmarks: Vec::new(),
            user_posts: HashMap::new(),
            user_likes_posts: HashMap::new(),
            user_view_states: HashMap::new(),
            media_image_sizes: HashMap::new(),
            timeline_cursor: None,
            user_cursors: HashMap::new(),
            user_likes_cursors: HashMap::new(),
            post_cache: HashMap::new(),
            post_cache_order: VecDeque::new(),
            scroll_to_top: false,
            user_infos_cache: HashMap::new(),
            image_cache: HashMap::new(),
            post_likers_cache: HashMap::new(),
            post_reposters_cache: HashMap::new(),
            post_replies_cache: HashMap::new(),
            opened_image_views: HashSet::new(),
            opened_raw_views: HashMap::new(),
            download_tasks: HashMap::new(),
            next_download_id: 0,
            is_search_window_open: is_screenshot_mode,
            search_query: String::new(),
            search_results: Vec::new(),
            is_search_posts_window_open: false,
            search_posts_query: String::new(),
            search_posts_results: None,
            search_posts_cursor: None,
            unread_notifications: 0,
            notifications: Vec::new(),
            settings: AppSettings::load(),
            is_settings_window_open: is_screenshot_mode,
            new_post_images: Vec::new(),
            reply_to: None,
            screenshot_requested: is_screenshot_mode,
            screenshot_output_path,
            frames_rendered: 0,
            screenshot_state,
        }
    }
}
impl RedskyApp {
    fn post_message(&self, msg: BskyActorMsg) {
        let _ = self.tx.send(msg);
    }
    fn post_ui_message(&self, msg: RedskyUiMsg) {
        let _ = self.ui_tx.send(msg);
    }
    fn request_image(&mut self, img_url: &str) {
        if img_url.is_empty() {
            return;
        }
        if self.image_cache.get(img_url).is_none() {
            println!("requesting image {}", img_url);
            self.image_cache.insert(img_url.to_string(), None);
            self.post_message(BskyActorMsg::LoadImage {
                url: img_url.to_string(),
            });
        }
    }
    fn request_post_images(&mut self, posts: &Vec<Post>) {
        for post in posts {
            self.request_image(&post.avatar_img);
            if let Some(quoted_post) = &post.quoted_post {
                self.request_image(&quoted_post.avatar_img);
            }
            for embed in &post.embeds {
                self.request_image(&embed.thumbnail_url);
            }
        }
    }
    fn update_post_optimistically<F>(&mut self, post_uri: &str, mut update_fn: F)
    where
        F: FnMut(&mut Post),
    {
        // Update timeline
        for item in &mut self.timeline {
            if let FeedItem::Full(post) = item {
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
        // Update user posts
        for posts in self.user_posts.values_mut().flatten() {
            for item in posts {
                if let FeedItem::Full(post) = item {
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
        // Update user likes posts
        for posts in self.user_likes_posts.values_mut().flatten() {
            for item in posts {
                if let FeedItem::Full(post) = item {
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
        for posts in self.post_replies_cache.values_mut().flatten() {
            for item in posts {
                if let FeedItem::Full(post) = item {
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
        // Update bookmarks
        for post in &mut self.bookmarks {
            if post.uri == post_uri {
                update_fn(post);
            }
            if let Some(quoted) = &mut post.quoted_post {
                if quoted.uri == post_uri {
                    update_fn(quoted);
                }
            }
        }
        // Update post cache
        if let Some(post) = self.post_cache.get_mut(post_uri) {
            update_fn(post);
        }
        for post in self.post_cache.values_mut() {
            if let Some(quoted) = &mut post.quoted_post {
                if quoted.uri == post_uri {
                    update_fn(quoted);
                }
            }
        }
    }
}
impl eframe::App for RedskyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();

        self.frames_rendered += 1;

        if let Some(state) = self.screenshot_state {
            if self.frames_rendered == 1 {
                match state {
                    ScreenshotState::Timeline => {
                        self.main_view_state = MainViewState::TimelineFeed;
                        self.is_settings_window_open = false;
                        self.is_post_window_open = false;
                    }
                    ScreenshotState::Profile => {
                        self.main_view_state = MainViewState::OwnPostFeed;
                    }
                    ScreenshotState::Bookmarks => {
                        self.main_view_state = MainViewState::BookmarksFeed;
                    }
                    ScreenshotState::Notifications => {
                        self.main_view_state = MainViewState::NotificationsFeed;
                    }
                    ScreenshotState::Thread => {
                        self.main_view_state = MainViewState::TimelineFeed;
                        self.post_replies_cache.insert(
                            StrongRef { uri: "at://mock-uri".to_string(), cid: "bafyreidfzuflltehrwqx5dzlqg3vzd2q6fudx75h7m3e7y22qpxg3ntv6m".parse().unwrap() },
                            Some(vec![FeedItem::Full(Post {
                                uri: "at://mock-reply-uri".to_string(),
                                cid: "bafyreidfzuflltehrwqx5dzlqg3vzd2q6fudx75h7m3e7y22qpxg3ntv6m".parse().unwrap(),
                                content: "Mock post thread reply content".to_string(),
                                author: "mockauthor2.bsky.social".to_string(),
                                display_name: "Mock Author 2".to_string(),
                                avatar_img: "".to_string(),
                                date: "2024-01-01T00:01:00Z".to_string(),
                                like_count: 1,
                                repost_count: 0,
                                embeds: vec![],
                                quoted_post: None,
                                is_reply: true,
                                viewer_like: None,
                                viewer_repost: None,
                                thread_root: Some(StrongRef { uri: "at://mock-uri".to_string(), cid: "bafyreidfzuflltehrwqx5dzlqg3vzd2q6fudx75h7m3e7y22qpxg3ntv6m".parse().unwrap() }),
                                raw_json: "{}".to_string(),
                            })]),
                        );
                    }
                    ScreenshotState::Settings => {
                        self.post_replies_cache.clear();
                        self.is_settings_window_open = true;
                    }
                    ScreenshotState::NewPost => {
                        self.is_settings_window_open = false;
                        self.is_post_window_open = true;
                    }
                    ScreenshotState::Done => {
                        std::process::exit(0);
                    }
                }
            }

            if self.screenshot_requested && self.frames_rendered > 10 {
                self.screenshot_requested = false;
                if let Some(vid) = state.viewport_id() {
                    ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::Screenshot(egui::UserData::default()));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                }
            }

            if self.screenshot_requested {
                ctx.request_repaint();
            }
        }

        for ev in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = ev {
                if let Some(state) = self.screenshot_state {
                    let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                    let output_path = state.filename();
                    image::save_buffer(
                        output_path,
                        &pixels,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    ).expect("Failed to save screenshot");

                    self.screenshot_state = Some(state.next());
                    self.screenshot_requested = true;
                    self.frames_rendered = 0;
                    ctx.request_repaint();
                }
            }
        }

        ctx.set_pixels_per_point(self.settings.zoom_factor);
        while let Ok(msg) = self.rx.try_recv() {
            self.process_message(ctx, msg);
        }
        self.make_user_timelines_views(ctx);
        self.make_download_progress_view(ctx);
        self.make_image_viewports(ctx);
        self.make_open_thread_views(ctx);
        self.make_user_list_viewports(ctx);
        if self.is_post_window_open {
            self.make_new_post_view(ctx);
        }
        if self.is_search_window_open {
            self.make_search_window(ctx);
        }
        if self.is_search_posts_window_open {
            self.make_search_posts_window(ctx);
        }
        if self.is_settings_window_open {
            self.make_settings_window(ctx);
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
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New post...").clicked() {
                            self.is_post_window_open = true;
                            self.reply_to = None;
                        }
                        if ui.button("Search accounts...").clicked() {
                            self.is_search_window_open = true;
                        }
                        if ui.button("Search posts...").clicked() {
                            self.is_search_posts_window_open = true;
                        }
                        if ui.button("Settings...").clicked() {
                            self.is_settings_window_open = true;
                        }
                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });
                });
                if self.main_view_state != MainViewState::Login {
                    ui.vertical(|ui| {
                        ui.horizontal_top(|ui| {
                            ui.selectable_value(
                                &mut self.main_view_state,
                                MainViewState::OwnPostFeed,
                                RichText::new("Profile").heading(),
                            );
                            ui.selectable_value(
                                &mut self.main_view_state,
                                MainViewState::TimelineFeed,
                                RichText::new("Timeline feed").heading(),
                            );
                            ui.selectable_value(
                                &mut self.main_view_state,
                                MainViewState::BookmarksFeed,
                                RichText::new("Bookmarks").heading(),
                            );
                            let bell_icon = if self.unread_notifications > 0 {
                                " 🔔"
                            } else {
                                ""
                            };
                            ui.selectable_value(
                                &mut self.main_view_state,
                                MainViewState::NotificationsFeed,
                                RichText::new(format!("Notifications{}", bell_icon)).heading(),
                            );
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
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.pass).password(true),
                                    )
                                    .labelled_by(pwd_label.id);
                                });
                                ui.checkbox(&mut self.remember_me, "Remember me");
                                ui.horizontal(|ui| {
                                    if ui.button("login").clicked() {
                                        self.post_message(BskyActorMsg::Login {
                                            login: self.login.to_string(),
                                            pass: self.pass.to_string(),
                                        });
                                    }
                                });
                            });
                        });
                    }
                    MainViewState::TimelineFeed => {
                        ui.with_layout(
                            egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true),
                            |ui| {
                                ui.vertical(|ui| {
                                    let mut timeline = std::mem::take(&mut self.timeline);
                                    self.make_post_view(ui, "Your timeline", &mut timeline);
                                    self.timeline = timeline;
                                });
                            },
                        );
                    }
                    MainViewState::BookmarksFeed => {
                        ui.with_layout(
                            egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true),
                            |ui| {
                                ui.vertical(|ui| {
                                    // Bookmarks also needs to use FeedItem if I want to use make_post_view
                                    // or I should convert them.
                                    // In this patch I'll convert them for simplicity as a first step.
                                    let mut bookmark_items = crate::app::into_feed_items(
                                        self.bookmarks.iter().cloned()
                                    );
                                    self.make_post_view(ui, "Your bookmarks", &mut bookmark_items);
                                    // Note: changes to bookmark_items (like dehydration) won't persist back to self.bookmarks
                                    // this way. Ideally bookmarks should also be Vec<FeedItem>.
                                });
                            },
                        );
                    }
                    MainViewState::NotificationsFeed => {
                        ui.with_layout(
                            egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true),
                            |ui| {
                                ui.vertical(|ui| {
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        for notif in &self.notifications {
                                            ui.horizontal(|ui| {
                                                if !notif.author_avatar.is_empty() {
                                                    if let Some(texture) = self
                                                        .image_cache
                                                        .get(&notif.author_avatar)
                                                        .unwrap_or(&None)
                                                    {
                                                        ui.add(
                                                            egui::Image::new(texture)
                                                                .max_width(24.0)
                                                                .max_height(24.0),
                                                        );
                                                    } else {
                                                        ui.spinner();
                                                    }
                                                }
                                                let action_text = match notif.reason.as_str() {
                                                    "like" => "liked your post",
                                                    "repost" => "reposted your post",
                                                    "follow" => "followed you",
                                                    "mention" => "mentioned you",
                                                    "reply" => "replied to your post",
                                                    "quote" => "quoted your post",
                                                    _ => &notif.reason,
                                                };
                                                ui.label(
                                                    RichText::new(format!(
                                                        "@{} {}",
                                                        notif.author, action_text
                                                    ))
                                                    .strong(),
                                                );
                                                if !notif.is_read {
                                                    ui.label(
                                                        RichText::new("🔴 Unread")
                                                            .color(egui::Color32::RED),
                                                    );
                                                }
                                            });
                                            ui.separator();
                                        }
                                    });
                                    if ui.button("Refresh notifications").clicked() {
                                        self.post_message(BskyActorMsg::GetNotifications());
                                    }
                                });
                            },
                        );
                    }
                    MainViewState::OwnPostFeed => {
                        let login = self.login.clone();
                        let mut maybe_post = self.user_posts.get_mut(&login).and_then(|p| p.take());
                        let mut maybe_likes = self.user_likes_posts.get_mut(&login).and_then(|p| p.take());
                        self.make_maybe_user_post_view(ui, &login, &mut maybe_post, &mut maybe_likes);
                        self.user_posts.insert(login.clone(), maybe_post);
                        self.user_likes_posts.insert(login, maybe_likes);
                    }
                }
            })
        });
    }
}
include!("ui_post.rs");
include!("ui_user.rs");
include!("ui_thread.rs");
include!("ui_widgets.rs");
include!("msg_handler.rs");
include!("ui_settings.rs");
