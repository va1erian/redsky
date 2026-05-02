impl BskyJob {
    async fn get_post_likers(
        &self,
        strong_ref: &StrongRef,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let likers = vec![UserProfile {
            handle: "testliker.bsky.social".to_string(),
            display_name: "Test Liker".to_string(),
            bio: "I like tests".to_string(),
            avatar_uri: "".to_string(),
            follower_count: 5,
            follow_count: 5,
            post_count: 5,
        }];
        Ok(RedskyUiMsg::NotifyLikesLoaded {
            post_uri: strong_ref.clone(),
            likers,
            cursor: None,
            append: false,
        })
    }

    async fn get_post_reposted_by(
        &self,
        strong_ref: &StrongRef,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let reposters = vec![UserProfile {
            handle: "testreposter.bsky.social".to_string(),
            display_name: "Test Reposter".to_string(),
            bio: "I repost tests".to_string(),
            avatar_uri: "".to_string(),
            follower_count: 5,
            follow_count: 5,
            post_count: 5,
        }];
        Ok(RedskyUiMsg::NotifyRepostersLoaded {
            post_uri: strong_ref.clone(),
            reposters,
            cursor: None,
            append: false,
        })
    }

    async fn like(
        &self,
        strong_ref: StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::NotifyLikeActionSucceeded {
            post_uri: strong_ref.uri.clone(),
            like_uri: "mock-like-uri".to_string(),
        })
    }

    async fn unlike(
        &self,
        _post_uri: String,
        _like_record_uri: String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn repost(
        &self,
        strong_ref: StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::NotifyRepostActionSucceeded {
            post_uri: strong_ref.uri.clone(),
            repost_uri: "mock-repost-uri".to_string(),
        })
    }

    async fn unrepost(
        &self,
        _post_uri: String,
        _repost_record_uri: String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn get_post_thread(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let post = Post {
            uri: strong_ref.uri.clone(),
            cid: strong_ref.cid.clone(),
            content: "Mock post thread root content".to_string(),
            author: "mockauthor.bsky.social".to_string(),
            display_name: "Mock Author".to_string(),
            avatar_img: "".to_string(),
            date: "2024-01-01T00:00:00Z".to_string(),
            like_count: 10,
            repost_count: 5,
            embeds: vec![],
            quoted_post: None,
            is_reply: false,
            viewer_like: None,
            viewer_repost: None,
            thread_root: None,
            raw_json: "{}".to_string(),
        };
        let reply = Post {
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
            thread_root: Some(strong_ref.clone()),
            raw_json: "{}".to_string(),
        };
        Ok(RedskyUiMsg::NotifyPostAndRepliesLoaded {
            post,
            replies: vec![reply],
        })
    }

    async fn get_raw_post(
        &self,
        post_uri: String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ShowRawPostView {
            post_uri,
            raw_json: "{}".to_string(),
        })
    }

    async fn search_posts(
        &self,
        _query: &String,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ShowSearchPostsResults {
            posts: vec![],
            cursor: None,
            append: false,
        })
    }

    async fn search_actors(
        &self,
        _query: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ShowSearchResults { results: vec![] })
    }

    async fn get_user_likes(
        &self,
        username: &String,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ShowUserLikesMsg {
            username: username.clone(),
            posts: vec![],
            cursor: None,
            append: false,
        })
    }

    async fn get_user_posts(
        &self,
        username: &String,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ShowUserPostsMsg {
            username: username.clone(),
            posts: vec![],
            cursor: None,
            append: false,
        })
    }

    async fn get_user_profile(
        &self,
        username: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ShowUserProfile {
            profile: UserProfile {
                handle: username.clone(),
                display_name: "Mock User".to_string(),
                bio: "Mock profile bio".to_string(),
                avatar_uri: "".to_string(),
                follower_count: 100,
                follow_count: 50,
                post_count: 200,
            },
        })
    }

    async fn get_timeline_posts(
        &self,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::RefreshTimelineMsg {
            posts: vec![],
            cursor: None,
            append: false,
        })
    }

    async fn get_bookmarks(&self, _cursor: &Option<String>) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: vec![],
            cursor: None,
            append: false,
        })
    }

    async fn login(
        &self,
        _login: &String,
        _pass: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::LogInSucceededMsg())
    }

    async fn download_all_images(
        &self,
        _id: u64,
        _username: &String,
        _path: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn get_unread_count(
        &self,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::NotifyUnreadCount { count: 3 })
    }

    async fn get_notifications(
        &self,
        _cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::RefreshNotificationsMsg {
            notifications: vec![],
            cursor: None,
            append: false,
        })
    }

    async fn load_image(
        &self,
        url: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let size = [1, 1];
        let pixels = [egui::Color32::from_rgb(255, 0, 0)];
        let color_image = egui::ColorImage::new(size, pixels.to_vec());

        Ok(RedskyUiMsg::NotifyImageLoaded {
            url: url.clone(),
            data: color_image,
        })
    }

    async fn post(
        &self,
        _msg_body: &String,
        _image_paths: &Vec<String>,
        _reply_to: &Option<(StrongRef, StrongRef)>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn delete_post(
        &self,
        _post_uri: String,
        _post_cid: Cid,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RedskyUiMsg::ActionSucceeded())
    }
}
