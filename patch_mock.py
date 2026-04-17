import re

content = open("src/bsky_actor/actor_methods_mock.rs").read()

content = content.replace(
    """    async fn get_post_likers(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_post_likers(
        &self,
        strong_ref: &StrongRef,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """        Ok(RedskyUiMsg::NotifyLikesLoaded {
            post_uri: strong_ref.clone(),
            likers,""",
    """        Ok(RedskyUiMsg::NotifyLikesLoaded {
            post_uri: strong_ref.clone(),
            likers,
            cursor: None,
            append: false,"""
)

content = content.replace(
    """    async fn get_post_reposted_by(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_post_reposted_by(
        &self,
        strong_ref: &StrongRef,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """        Ok(RedskyUiMsg::NotifyRepostersLoaded {
            post_uri: strong_ref.clone(),
            reposters,""",
    """        Ok(RedskyUiMsg::NotifyRepostersLoaded {
            post_uri: strong_ref.clone(),
            reposters,
            cursor: None,
            append: false,"""
)

content = content.replace(
    """    async fn get_bookmarks(&self) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_bookmarks(&self, _cursor: &Option<String>) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: vec![],
        })""",
    """        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: vec![],
            cursor: None,
            append: false,
        })"""
)

content = content.replace(
    """    async fn get_notifications(
        &self,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_notifications(
        &self,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """        Ok(RedskyUiMsg::RefreshNotificationsMsg {
            notifications: vec![],
        })""",
    """        Ok(RedskyUiMsg::RefreshNotificationsMsg {
            notifications: vec![],
            cursor: None,
            append: false,
        })"""
)

open("src/bsky_actor/actor_methods_mock.rs", "w").write(content)
