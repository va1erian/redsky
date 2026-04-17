import re

content = open("src/bsky_actor/mod.rs").read()

content = content.replace(
    "BskyActorMsg::GetPostLikers { post_ref } => self.get_post_likers(post_ref).await,",
    "BskyActorMsg::GetPostLikers { post_ref, cursor } => self.get_post_likers(post_ref, cursor).await,"
).replace(
    "BskyActorMsg::GetPostRepostedBy { post_ref } => {",
    "BskyActorMsg::GetPostRepostedBy { post_ref, cursor } => {"
).replace(
    "self.get_post_reposted_by(post_ref).await",
    "self.get_post_reposted_by(post_ref, cursor).await"
).replace(
    "BskyActorMsg::GetBookmarks() => self.get_bookmarks().await,",
    "BskyActorMsg::GetBookmarks { cursor } => self.get_bookmarks(cursor).await,"
).replace(
    "BskyActorMsg::GetNotifications() => self.get_notifications().await,",
    "BskyActorMsg::GetNotifications { cursor } => self.get_notifications(cursor).await,"
)

open("src/bsky_actor/mod.rs", "w").write(content)
