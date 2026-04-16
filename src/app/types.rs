#[derive(Eq, PartialEq, Hash, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StrongRef {
    pub uri: String,
    pub cid: Cid,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UserProfile {
    pub handle: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_uri: String,
    pub follower_count: i64,
    pub follow_count: i64,
    pub post_count: i64,
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
    pub viewer_like: Option<String>,
    pub viewer_repost: Option<String>,
    pub thread_root: Option<StrongRef>,
    pub raw_json: String,
}
pub enum FeedItem {
    Full(Post),
    Dehydrated { uri: String },
}

pub fn into_feed_items(posts: impl IntoIterator<Item = Post>) -> Vec<FeedItem> {
    posts.into_iter().map(FeedItem::Full).collect()
}

#[derive(Debug)]
pub struct DownloadTask {
    #[allow(dead_code)]
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
    pub alt: String,
}
impl PostImage {
    pub fn new(thumb: String, url: String, alt: String) -> Self {
        PostImage {
            thumbnail_url: thumb,
            url,
            alt,
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
#[derive(Clone, Debug)]
pub struct AppNotification {
    #[allow(dead_code)]
    pub uri: String,
    pub author: String,
    pub author_avatar: String,
    pub reason: String,
    pub is_read: bool,
}
pub enum RedskyUiMsg {
    LogInSucceededMsg(),
    ActionSucceeded(),
    RefreshBookmarksMsg {
        posts: Vec<Post>,
    },
    PrepareUserView {
        username: String,
    },
    PrepareThreadView {
        thread_ref: StrongRef,
    },
    CloseThreadView {
        thread_ref: StrongRef,
    },
    NotifyImageLoaded {
        url: String,
        data: egui::ColorImage,
    },
    #[allow(dead_code)]
    NotifyLikesLoaded {
        post_uri: StrongRef,
        likers: Vec<UserProfile>,
    },
    NotifyRepostersLoaded {
        post_uri: StrongRef,
        reposters: Vec<UserProfile>,
    },
    CloseLikesView {
        post_uri: StrongRef,
    },
    CloseRepostersView {
        post_uri: StrongRef,
    },
    PrepareLikeAction {
        post_uri: String,
        post_cid: Cid,
        unlike: bool,
    },
    PrepareRepostAction {
        post_uri: String,
        post_cid: Cid,
        unrepost: bool,
    },
    NotifyLikeActionSucceeded {
        post_uri: String,
        like_uri: String,
    },
    NotifyRepostActionSucceeded {
        post_uri: String,
        repost_uri: String,
    },
    NotifyPostAndRepliesLoaded {
        post: Post,
        replies: Vec<Post>,
    },
    ShowUserProfile {
        profile: UserProfile,
    },
    RefreshTimelineMsg {
        posts: Vec<Post>,
        cursor: Option<String>,
        append: bool,
    },
    ShowUserPostsMsg {
        username: String,
        posts: Vec<Post>,
        cursor: Option<String>,
        append: bool,
    },
    ShowUserLikesMsg {
        username: String,
        posts: Vec<Post>,
        cursor: Option<String>,
        append: bool,
    },
    DropUserPostsMsg {
        username: String,
    },
    PrepareImageView {
        img_uri: String,
    },
    ShowBigImageView {
        img_uri: String,
    },
    CloseBigImageView {
        img_uri: String,
    },
    ShowRawPostView {
        post_uri: String,
        raw_json: String,
    },
    CloseRawPostView {
        post_uri: String,
    },
    DeletePost {
        post_uri: String,
        post_cid: Cid,
    },
    ShowErrorMsg {
        error: String,
    },
    DownloadProgress {
        id: u64,
        processed_posts: usize,
        total_posts: Option<usize>,
        downloaded_images: usize,
        total_images: Option<usize>,
        status: DownloadStatus,
    },
    DownloadFinished {
        id: u64,
        errors: Vec<String>,
    },
    StartDownloadJob {
        username: String,
        path: String,
    },
    ShowSearchResults {
        results: Vec<UserProfile>,
    },
    ShowSearchPostsResults {
        posts: Vec<Post>,
        cursor: Option<String>,
        append: bool,
    },
    NotifyUnreadCount {
        count: i64,
    },
    RefreshNotificationsMsg {
        notifications: Vec<AppNotification>,
    },
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BskyActorMsg {
    Login {
        login: String,
        pass: String,
    },
    Post {
        msg_body: String,
        image_paths: Vec<String>,
        reply_to: Option<(StrongRef, StrongRef)>,
    },
    GetTimeline {
        cursor: Option<String>,
    },
    GetBookmarks(),
    Like {
        post_ref: StrongRef,
    },
    Unlike {
        post_uri: String,
        like_record_uri: String,
    },
    DeletePost {
        post_uri: String,
        post_cid: Cid,
    },
    Repost {
        post_ref: StrongRef,
    },
    Unrepost {
        post_uri: String,
        repost_record_uri: String,
    },
    GetPostLikers {
        post_ref: StrongRef,
    },
    GetPostRepostedBy {
        post_ref: StrongRef,
    },
    GetPostAndReplies {
        post_ref: StrongRef,
    },
    GetUserProfile {
        username: String,
    },
    GetUserPosts {
        username: String,
        cursor: Option<String>,
    },
    GetUserLikes {
        username: String,
        cursor: Option<String>,
    },
    SearchActors {
        query: String,
    },
    SearchPosts {
        query: String,
        cursor: Option<String>,
    },
    LoadImage {
        url: String,
    },
    StartImageDownload {
        id: u64,
        username: String,
        path: String,
    },
    CancelImageDownload {
        id: u64,
    },
    GetUnreadCount(),
    GetNotifications(),
    #[allow(dead_code)]
    Close(),
}
#[derive(PartialEq, Eq, Clone, Debug)]
enum MainViewState {
    Login,
    TimelineFeed,
    OwnPostFeed,
    BookmarksFeed,
    NotificationsFeed,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum UserViewState {
    Posts,
    Media,
    Liked,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AppTheme {
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub theme: AppTheme,
    pub max_image_size: f32,
    pub zoom_factor: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: AppTheme::System,
            max_image_size: 640.0,
            zoom_factor: 1.0,
        }
    }
}



impl AppSettings {
    pub fn load() -> Self {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "Redsky", "Redsky") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("settings.toml");
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(settings) = toml::from_str(&contents) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "Redsky", "Redsky") {
            let config_dir = proj_dirs.config_dir();
            if std::fs::create_dir_all(config_dir).is_ok() {
                let config_path = config_dir.join("settings.toml");
                if let Ok(contents) = toml::to_string(self) {
                    let _ = std::fs::write(config_path, contents);
                }
            }
        }
    }
}
