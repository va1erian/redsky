#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotState {
    Timeline,
    Profile,
    Bookmarks,
    Notifications,
    Thread,
    Settings,
    NewPost,
    Done,
}

impl ScreenshotState {
    pub fn next(&self) -> Self {
        match self {
            Self::Timeline => Self::Profile,
            Self::Profile => Self::Bookmarks,
            Self::Bookmarks => Self::Notifications,
            Self::Notifications => Self::Thread,
            Self::Thread => Self::Settings,
            Self::Settings => Self::NewPost,
            Self::NewPost => Self::Done,
            Self::Done => Self::Done,
        }
    }

    pub fn filename(&self) -> &'static str {
        match self {
            Self::Timeline => "screenshot_timeline.png",
            Self::Profile => "screenshot_profile.png",
            Self::Bookmarks => "screenshot_bookmarks.png",
            Self::Notifications => "screenshot_notifications.png",
            Self::Thread => "screenshot_thread.png",
            Self::Settings => "screenshot_settings.png",
            Self::NewPost => "screenshot_new_post.png",
            Self::Done => "done.png",
        }
    }

    pub fn viewport_id(&self) -> Option<egui::ViewportId> {
        match self {
            Self::Thread => Some(egui::ViewportId::from_hash_of("at://mock-uri")),
            Self::Settings => Some(egui::ViewportId::from_hash_of("__settings")),
            Self::NewPost => Some(egui::ViewportId::from_hash_of("__new_post")),
            _ => None, // Use root viewport for timeline, profile, bookmarks, notifications
        }
    }
}
