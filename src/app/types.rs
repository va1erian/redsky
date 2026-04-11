#[derive(Eq, PartialEq, Hash, Clone, Debug)]
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
}
pub enum FeedItem {
    Full(Post),
    Dehydrated { uri: String },
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
    SearchActors {
        query: String,
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