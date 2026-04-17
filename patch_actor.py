import re

content = open("src/bsky_actor/actor_methods.rs").read()

content = content.replace(
    """    async fn get_post_likers(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_post_likers(
        &self,
        strong_ref: &StrongRef,
        cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """                atrium_api::app::bsky::feed::get_likes::ParametersData {
                    cid: Some(strong_ref.cid.clone()),
                    uri: strong_ref.uri.clone(),
                    cursor: None,
                    limit: None,
                }""",
    """                atrium_api::app::bsky::feed::get_likes::ParametersData {
                    cid: Some(strong_ref.cid.clone()),
                    uri: strong_ref.uri.clone(),
                    cursor: cursor.clone(),
                    limit: None,
                }"""
).replace(
    """        Ok(RedskyUiMsg::NotifyLikesLoaded {
            post_uri: strong_ref.clone(),
            likers,""",
    """        Ok(RedskyUiMsg::NotifyLikesLoaded {
            post_uri: strong_ref.clone(),
            likers,
            cursor: response.data.cursor,
            append: cursor.is_some(),"""
)

content = content.replace(
    """    async fn get_post_reposted_by(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_post_reposted_by(
        &self,
        strong_ref: &StrongRef,
        cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """                atrium_api::app::bsky::feed::get_reposted_by::ParametersData {
                    cid: Some(strong_ref.cid.clone()),
                    uri: strong_ref.uri.clone(),
                    cursor: None,
                    limit: None,
                }""",
    """                atrium_api::app::bsky::feed::get_reposted_by::ParametersData {
                    cid: Some(strong_ref.cid.clone()),
                    uri: strong_ref.uri.clone(),
                    cursor: cursor.clone(),
                    limit: None,
                }"""
).replace(
    """        Ok(RedskyUiMsg::NotifyRepostersLoaded {
            post_uri: strong_ref.clone(),
            reposters,""",
    """        Ok(RedskyUiMsg::NotifyRepostersLoaded {
            post_uri: strong_ref.clone(),
            reposters,
            cursor: response.data.cursor,
            append: cursor.is_some(),"""
)

content = content.replace(
    """    async fn get_bookmarks(&self) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_bookmarks(&self, cursor: &Option<String>) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """                atrium_api::app::bsky::bookmark::get_bookmarks::ParametersData {
                    cursor: None,
                    limit: None,
                }""",
    """                atrium_api::app::bsky::bookmark::get_bookmarks::ParametersData {
                    cursor: cursor.clone(),
                    limit: None,
                }"""
).replace(
    """        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: response
                .data
                .bookmarks
                .iter()
                .flat_map(extract_post_from_bookmark)
                .collect(),
        })""",
    """        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: response
                .data
                .bookmarks
                .iter()
                .flat_map(extract_post_from_bookmark)
                .collect(),
            cursor: response.data.cursor,
            append: cursor.is_some(),
        })"""
)

content = content.replace(
    """    async fn get_notifications(
        &self,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {""",
    """    async fn get_notifications(
        &self,
        cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {"""
).replace(
    """                atrium_api::app::bsky::notification::list_notifications::ParametersData {
                    cursor: None,
                    limit: None,
                }""",
    """                atrium_api::app::bsky::notification::list_notifications::ParametersData {
                    cursor: cursor.clone(),
                    limit: None,
                }"""
).replace(
    """        Ok(RedskyUiMsg::RefreshNotificationsMsg { notifications })""",
    """        Ok(RedskyUiMsg::RefreshNotificationsMsg {
            notifications,
            cursor: response.data.cursor,
            append: cursor.is_some(),
        })"""
)

open("src/bsky_actor/actor_methods.rs", "w").write(content)
