import re

content = open("src/app/msg_handler.rs").read()

content = content.replace(
    """            RedskyUiMsg::RefreshBookmarksMsg { posts } => {
                self.request_post_images(&posts);
                self.bookmarks = posts;
            }""",
    """            RedskyUiMsg::RefreshBookmarksMsg { posts, cursor, append } => {
                self.request_post_images(&posts);
                if append {
                    self.bookmarks.extend(posts);
                } else {
                    self.bookmarks = posts;
                }
                self.bookmarks_cursor = cursor;
            }"""
)

content = content.replace(
    """            RedskyUiMsg::NotifyLikesLoaded { post_uri, likers } => {
                self.post_likers_cache.insert(post_uri, likers);
            }""",
    """            RedskyUiMsg::NotifyLikesLoaded { post_uri, likers, cursor, append } => {
                if append {
                    if let Some((existing, existing_cursor)) = self.post_likers_cache.get_mut(&post_uri) {
                        existing.extend(likers);
                        *existing_cursor = cursor;
                    }
                } else {
                    self.post_likers_cache.insert(post_uri, (likers, cursor));
                }
            }"""
)

content = content.replace(
    """            RedskyUiMsg::NotifyRepostersLoaded {
                post_uri,
                reposters,
            } => {
                self.post_reposters_cache.insert(post_uri, reposters);
            }""",
    """            RedskyUiMsg::NotifyRepostersLoaded {
                post_uri,
                reposters,
                cursor,
                append,
            } => {
                if append {
                    if let Some((existing, existing_cursor)) = self.post_reposters_cache.get_mut(&post_uri) {
                        existing.extend(reposters);
                        *existing_cursor = cursor;
                    }
                } else {
                    self.post_reposters_cache.insert(post_uri, (reposters, cursor));
                }
            }"""
)

content = content.replace(
    """            RedskyUiMsg::RefreshNotificationsMsg { notifications } => {
                for notif in &notifications {
                    self.request_image(&notif.author_avatar);
                }
                self.notifications = notifications;
            }""",
    """            RedskyUiMsg::RefreshNotificationsMsg { notifications, cursor, append } => {
                for notif in &notifications {
                    self.request_image(&notif.author_avatar);
                }
                if append {
                    self.notifications.extend(notifications);
                } else {
                    self.notifications = notifications;
                }
                self.notifications_cursor = cursor;
            }"""
)

content = content.replace(
    "self.post_message(BskyActorMsg::GetBookmarks());",
    "self.post_message(BskyActorMsg::GetBookmarks { cursor: None });"
)
content = content.replace(
    "self.post_message(BskyActorMsg::GetNotifications());",
    "self.post_message(BskyActorMsg::GetNotifications { cursor: None });"
)

open("src/app/msg_handler.rs", "w").write(content)
