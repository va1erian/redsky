import re

content = open("src/app/types.rs").read()

content = re.sub(
    r"GetPostLikers {\s*post_ref: StrongRef,\s*}",
    "GetPostLikers {\n        post_ref: StrongRef,\n        cursor: Option<String>,\n    }",
    content
)
content = re.sub(
    r"GetPostRepostedBy {\s*post_ref: StrongRef,\s*}",
    "GetPostRepostedBy {\n        post_ref: StrongRef,\n        cursor: Option<String>,\n    }",
    content
)
content = re.sub(
    r"GetBookmarks\(\),",
    "GetBookmarks { cursor: Option<String> },",
    content
)
content = re.sub(
    r"GetNotifications\(\),",
    "GetNotifications { cursor: Option<String> },",
    content
)

content = re.sub(
    r"NotifyLikesLoaded {\s*post_uri: StrongRef,\s*likers: Vec<UserProfile>,\s*}",
    "NotifyLikesLoaded {\n        post_uri: StrongRef,\n        likers: Vec<UserProfile>,\n        cursor: Option<String>,\n        append: bool,\n    }",
    content
)
content = re.sub(
    r"NotifyRepostersLoaded {\s*post_uri: StrongRef,\s*reposters: Vec<UserProfile>,\s*}",
    "NotifyRepostersLoaded {\n        post_uri: StrongRef,\n        reposters: Vec<UserProfile>,\n        cursor: Option<String>,\n        append: bool,\n    }",
    content
)
content = re.sub(
    r"RefreshBookmarksMsg {\s*posts: Vec<Post>,\s*}",
    "RefreshBookmarksMsg {\n        posts: Vec<Post>,\n        cursor: Option<String>,\n        append: bool,\n    }",
    content
)
content = re.sub(
    r"RefreshNotificationsMsg {\s*notifications: Vec<AppNotification>,\s*}",
    "RefreshNotificationsMsg {\n        notifications: Vec<AppNotification>,\n        cursor: Option<String>,\n        append: bool,\n    }",
    content
)

open("src/app/types.rs", "w").write(content)
