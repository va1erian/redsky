impl BskyJob {

    #[allow(dead_code)]
    async fn get_post_likers(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get likers");

        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .feed
            .get_likes(
                atrium_api::app::bsky::feed::get_likes::ParametersData {
                    cid: Some(strong_ref.cid.clone()),
                    uri: strong_ref.uri.clone(),
                    cursor: None,
                    limit: None,
                }
                .into(),
            )
            .await?;

        let likers = response
            .data
            .likes
            .iter()
            .map(|like_data| {
                let profile = &like_data.actor;
                UserProfile {
                    handle: profile.handle.clone().into(),
                    display_name: profile
                        .display_name
                        .clone()
                        .unwrap_or("(no display name)".to_string()),
                    bio: profile
                        .description
                        .clone()
                        .unwrap_or("(No bio)".to_string()),
                    avatar_uri: profile.avatar.clone().unwrap_or("".to_string()),
                    follower_count: 0,
                    follow_count: 0,
                    post_count: 0,
                }
            })
            .collect();

        Ok(RedskyUiMsg::NotifyLikesLoaded {
            post_uri: strong_ref.clone(),
            likers,
        })
    }

    async fn get_post_reposted_by(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get reposters");

        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .feed
            .get_reposted_by(
                atrium_api::app::bsky::feed::get_reposted_by::ParametersData {
                    cid: Some(strong_ref.cid.clone()),
                    uri: strong_ref.uri.clone(),
                    cursor: None,
                    limit: None,
                }
                .into(),
            )
            .await?;

        let reposters = response
            .data
            .reposted_by
            .iter()
            .map(|profile| UserProfile {
                handle: profile.handle.clone().into(),
                display_name: profile
                    .display_name
                    .clone()
                    .unwrap_or("(no display name)".to_string()),
                bio: profile
                    .description
                    .clone()
                    .unwrap_or("(No bio)".to_string()),
                avatar_uri: profile.avatar.clone().unwrap_or("".to_string()),
                follower_count: 0,
                follow_count: 0,
                post_count: 0,
            })
            .collect();

        Ok(RedskyUiMsg::NotifyRepostersLoaded {
            post_uri: strong_ref.clone(),
            reposters,
        })
    }

    async fn like(
        &self,
        strong_ref: StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("liking post");
        let post_uri = strong_ref.uri.clone();
        let response = self
            .bsky_agent
            .create_record(atrium_api::app::bsky::feed::like::RecordData {
                created_at: Datetime::now(),
                subject: atrium_api::com::atproto::repo::strong_ref::MainData {
                    cid: strong_ref.cid.clone(),
                    uri: strong_ref.uri.clone(),
                }
                .into(),
                via: None,
            })
            .await?;
        Ok(RedskyUiMsg::NotifyLikeActionSucceeded {
            post_uri,
            like_uri: response.data.uri,
        })
    }

    pub async fn delete_post(
        &self,
        post_uri: String,
        _post_cid: Cid,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("deleting post");
        let parts: Vec<&str> = post_uri.split('/').collect();
        let rkey = parts.last().ok_or("Invalid post record URI")?;

        let session = self.bsky_agent.api.com.atproto.server.get_session().await?;

        self.bsky_agent
            .api
            .com
            .atproto
            .repo
            .delete_record(
                atrium_api::com::atproto::repo::delete_record::InputData {
                    collection: "app.bsky.feed.post".parse()?,
                    repo: AtIdentifier::Did(session.data.did),
                    rkey: RecordKey::new(rkey.to_string()).map_err(|e| e.to_string())?,
                    swap_commit: None,
                    swap_record: None,
                }
                .into(),
            )
            .await?;
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn unlike(
        &self,
        _post_uri: String,
        like_record_uri: String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("unliking post");
        let parts: Vec<&str> = like_record_uri.split('/').collect();
        let rkey = parts.last().ok_or("Invalid like record URI")?;

        let session = self.bsky_agent.api.com.atproto.server.get_session().await?;

        self.bsky_agent
            .api
            .com
            .atproto
            .repo
            .delete_record(
                atrium_api::com::atproto::repo::delete_record::InputData {
                    collection: "app.bsky.feed.like".parse()?,
                    repo: AtIdentifier::Did(session.data.did),
                    rkey: RecordKey::new(rkey.to_string()).map_err(|e| e.to_string())?,
                    swap_commit: None,
                    swap_record: None,
                }
                .into(),
            )
            .await?;
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn repost(
        &self,
        strong_ref: StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("reposting post");
        let post_uri = strong_ref.uri.clone();
        let response = self
            .bsky_agent
            .create_record(atrium_api::app::bsky::feed::repost::RecordData {
                created_at: Datetime::now(),
                subject: atrium_api::com::atproto::repo::strong_ref::MainData {
                    cid: strong_ref.cid.clone(),
                    uri: strong_ref.uri.clone(),
                }
                .into(),
                via: None,
            })
            .await?;
        Ok(RedskyUiMsg::NotifyRepostActionSucceeded {
            post_uri,
            repost_uri: response.data.uri,
        })
    }

    async fn unrepost(
        &self,
        _post_uri: String,
        repost_record_uri: String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("unreposting post");
        let parts: Vec<&str> = repost_record_uri.split('/').collect();
        let rkey = parts.last().ok_or("Invalid repost record URI")?;

        let session = self.bsky_agent.api.com.atproto.server.get_session().await?;

        self.bsky_agent
            .api
            .com
            .atproto
            .repo
            .delete_record(
                atrium_api::com::atproto::repo::delete_record::InputData {
                    collection: "app.bsky.feed.repost".parse()?,
                    repo: AtIdentifier::Did(session.data.did),
                    rkey: RecordKey::new(rkey.to_string()).map_err(|e| e.to_string())?,
                    swap_commit: None,
                    swap_record: None,
                }
                .into(),
            )
            .await?;
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn get_post_thread(
        &self,
        strong_ref: &StrongRef,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get post thread");

        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .feed
            .get_post_thread(
                atrium_api::app::bsky::feed::get_post_thread::ParametersData {
                    uri: strong_ref.uri.clone(),
                    depth: 1.try_into().ok(),
                    parent_height: 0.try_into().ok(),
                }
                .into(),
            )
            .await?;

        if let atrium_api::types::Union::Refs(OutputThreadRefs::AppBskyFeedDefsThreadViewPost(
            post_data,
        )) = &response.data.thread
        {
            let replies = match &post_data.replies {
                Some(reply_list) => reply_list
                    .iter()
                    .flat_map(|reply| match reply {
                        Union::Refs(maybe_reply) => {
                            if let ThreadViewPostRepliesItem::ThreadViewPost(view) = maybe_reply {
                                extract_post(&view.post).into_iter().collect()
                            } else {
                                vec![]
                            }
                        }
                        Union::Unknown(_) => {
                            vec![]
                        }
                    })
                    .collect(),
                None => {
                    vec![]
                }
            };

            if let Some(post) = extract_post(&post_data.post) {
                Ok(RedskyUiMsg::NotifyPostAndRepliesLoaded { post, replies })
            } else {
                Err("Failed to parse main post record".into())
            }
        } else {
            Ok(RedskyUiMsg::ActionSucceeded())
        }
    }

    async fn load_image(
        &self,
        url: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let url_clone = url.clone();
        let resp = reqwest::get(url).await?;
        let bytes = resp.bytes().await?;
        let color_image = tokio::task::spawn_blocking(move || {
            let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.into_rgba8();
            let pixels = image_buffer.as_flat_samples();
            Ok::<egui::ColorImage, String>(egui::ColorImage::from_rgba_unmultiplied(
                size,
                pixels.as_slice(),
            ))
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(RedskyUiMsg::NotifyImageLoaded {
            url: url_clone,
            data: color_image,
        })
    }

    async fn search_posts(
        &self,
        query: &String,
        cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("search posts", &query);
        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .feed
            .search_posts(
                atrium_api::app::bsky::feed::search_posts::ParametersData {
                    q: query.clone(),
                    limit: 30.try_into().ok(),
                    cursor: cursor.clone(),
                    author: None,
                    domain: None,
                    lang: None,
                    mentions: None,
                    since: None,
                    sort: None,
                    tag: None,
                    until: None,
                    url: None,
                }
                .into(),
            )
            .await?;

        Ok(RedskyUiMsg::ShowSearchPostsResults {
            posts: response
                .data
                .posts
                .iter()
                .filter_map(|post_view| extract_post(post_view))
                .collect(),
            cursor: response.data.cursor,
            append: cursor.is_some(),
        })
    }

    async fn search_actors(
        &self,
        query: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("search actors", &query);
        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .actor
            .search_actors_typeahead(
                atrium_api::app::bsky::actor::search_actors_typeahead::ParametersData {
                    limit: 10.try_into().ok(),
                    q: Some(query.clone()),
                    term: None, // DEPRECATED: use 'q' instead.
                }
                .into(),
            )
            .await?;

        let results = response
            .data
            .actors
            .iter()
            .map(|actor| UserProfile {
                handle: actor.handle.to_string(),
                display_name: actor
                    .display_name
                    .clone()
                    .unwrap_or("(no display name)".to_string()),
                bio: "".to_string(),
                avatar_uri: actor.avatar.clone().unwrap_or("".to_string()),
                follower_count: 0,
                follow_count: 0,
                post_count: 0,
            })
            .collect();

        Ok(RedskyUiMsg::ShowSearchResults { results })
    }

    async fn get_user_likes(
        &self,
        username: &String,
        cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get user likes");
        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .feed
            .get_actor_likes(
                atrium_api::app::bsky::feed::get_actor_likes::ParametersData {
                    actor: AtIdentifier::Handle(
                        username
                            .parse()
                            .map_err(|e| format!("Invalid handle: {}", e))?,
                    ),
                    cursor: cursor.clone(),
                    limit: 30.try_into().ok(),
                }
                .into(),
            )
            .await?;

        Ok(RedskyUiMsg::ShowUserLikesMsg {
            username: username.to_string(),
            posts: response
                .data
                .feed
                .iter()
                .filter_map(
                    |post_el: &atrium_api::types::Object<
                        atrium_api::app::bsky::feed::defs::FeedViewPostData,
                    >| { extract_post(&post_el.post) },
                )
                .collect(),
            cursor: response.data.cursor,
            append: cursor.is_some(),
        })
    }

    async fn get_user_posts(
        &self,
        username: &String,
        cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get user posts");
        let at_uri = format!("at://{}", username);
        dbg!(&at_uri);
        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .feed
            .get_author_feed(
                atrium_api::app::bsky::feed::get_author_feed::ParametersData {
                    actor: AtIdentifier::Handle(
                        username
                            .parse()
                            .map_err(|e| format!("Invalid handle: {}", e))?,
                    ),
                    cursor: cursor.clone(),
                    filter: None,
                    include_pins: Some(true),
                    limit: 30.try_into().ok(),
                }
                .into(),
            )
            .await?;

        Ok(RedskyUiMsg::ShowUserPostsMsg {
            username: username.to_string(),
            posts: response
                .data
                .feed
                .iter()
                .filter_map(
                    |post_el: &atrium_api::types::Object<
                        atrium_api::app::bsky::feed::defs::FeedViewPostData,
                    >| { extract_post(&post_el.post) },
                )
                .collect(),
            cursor: response.data.cursor,
            append: cursor.is_some(),
        })
    }

    async fn get_user_profile(
        &self,
        username: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get user profile", &username);

        let profile = self
            .bsky_agent
            .api
            .app
            .bsky
            .actor
            .get_profile(
                atrium_api::app::bsky::actor::get_profile::ParametersData {
                    actor: AtIdentifier::Handle(
                        username
                            .parse()
                            .map_err(|e| format!("Invalid handle: {}", e))?,
                    ),
                }
                .into(),
            )
            .await?;

        Ok(RedskyUiMsg::ShowUserProfile {
            profile: UserProfile {
                handle: username.clone(),
                display_name: profile
                    .display_name
                    .clone()
                    .unwrap_or("(no display name)".to_string()),
                bio: profile
                    .description
                    .clone()
                    .unwrap_or("(No bio)".to_string()),
                avatar_uri: profile.avatar.clone().unwrap_or("".to_string()),
                follower_count: profile.followers_count.unwrap_or_default(),
                follow_count: profile.follows_count.unwrap_or_default(),
                post_count: profile.posts_count.unwrap_or_default(),
            },
        })
    }

    async fn get_timeline_posts(
        &self,
        cursor: &Option<String>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get tl");

        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .feed
            .get_timeline(
                atrium_api::app::bsky::feed::get_timeline::ParametersData {
                    algorithm: None,
                    cursor: cursor.clone(),
                    limit: 30.try_into().ok(),
                }
                .into(),
            )
            .await?;

        Ok(RedskyUiMsg::RefreshTimelineMsg {
            posts: response
                .data
                .feed
                .iter()
                .filter_map(|feed_element| extract_post(&feed_element.post))
                .collect(),
            cursor: response.data.cursor,
            append: cursor.is_some(),
        })
    }

    async fn get_bookmarks(&self) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("get bookmarks");

        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .bookmark
            .get_bookmarks(
                atrium_api::app::bsky::bookmark::get_bookmarks::ParametersData {
                    cursor: None,
                    limit: 30.try_into().ok(),
                }
                .into(),
            )
            .await?;

        Ok(RedskyUiMsg::RefreshBookmarksMsg {
            posts: response
                .data
                .bookmarks
                .iter()
                .flat_map(extract_post_from_bookmark)
                .collect(),
        })
    }

    async fn login(
        &self,
        login: &String,
        pass: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("loggin in");
        let _ = self.bsky_agent.login(login, pass).await?;
        Ok(RedskyUiMsg::LogInSucceededMsg())
    }

    async fn download_all_images(
        &self,
        id: u64,
        username: &String,
        path: &String,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_posts = Vec::new();
        let mut cursor = None;

        // 1. Scan all posts
        loop {
            let response = self
                .bsky_agent
                .api
                .app
                .bsky
                .feed
                .get_author_feed(
                    atrium_api::app::bsky::feed::get_author_feed::ParametersData {
                        actor: AtIdentifier::Handle(
                            username
                                .parse()
                                .map_err(|e| format!("Invalid handle: {}", e))?,
                        ),
                        cursor: cursor.clone(),
                        filter: None,
                        include_pins: Some(true),
                        limit: 100.try_into().ok(),
                    }
                    .into(),
                )
                .await?;

            let posts_in_batch: Vec<Post> = response
                .data
                .feed
                .iter()
                .filter_map(|post_el| extract_post(&post_el.post))
                .collect();

            all_posts.extend(posts_in_batch);
            cursor = response.data.cursor;

            self.post_to_ui(RedskyUiMsg::DownloadProgress {
                id,
                processed_posts: all_posts.len(),
                total_posts: None,
                downloaded_images: 0,
                total_images: None,
                status: DownloadStatus::Scanning,
            });

            if cursor.is_none() {
                break;
            }
            // Rate limiting awareness
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // 2. Filter images (only those posted by this account as requested, skip replies and reposts)
        let mut images_to_download = Vec::new();
        for post in &all_posts {
            // Note: atrium_api::app::bsky::feed::defs::FeedViewPostData also has 'reason' for reposts,
            // but here we filter by author and check if it's a reply.
            if post.author == *username && !post.is_reply {
                for img in &post.embeds {
                    images_to_download.push((img.url.clone(), post.date.clone()));
                }
            }
        }

        let total_images = images_to_download.len();
        self.post_to_ui(RedskyUiMsg::DownloadProgress {
            id,
            processed_posts: all_posts.len(),
            total_posts: Some(all_posts.len()),
            downloaded_images: 0,
            total_images: Some(total_images),
            status: DownloadStatus::Downloading,
        });

        // 3. Download images
        let mut downloaded_count = 0;
        let mut errors = Vec::new();
        let target_dir = std::path::Path::new(path).to_path_buf();

        let mut set = tokio::task::JoinSet::new();
        let concurrency_limit = 5;
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency_limit));

        for (url, date) in images_to_download {
            let semaphore = std::sync::Arc::clone(&semaphore);
            let target_dir = target_dir.clone();
            set.spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = async {
                    let resp = reqwest::get(&url).await?;
                    let bytes = resp.bytes().await?;

                    let raw_filename = url.split('/').next_back().unwrap_or("image");
                    let extension = if raw_filename.contains("@png") {
                        "png"
                    } else if raw_filename.contains("@jpeg") || raw_filename.contains("@jpg") {
                        "jpg"
                    } else if raw_filename.contains("@webp") {
                        "webp"
                    } else if raw_filename.contains("@gif") {
                        "gif"
                    } else {
                        "png"
                    };

                    let base_name = raw_filename.split('@').next().unwrap_or(raw_filename);
                    let truncated_base: String = if base_name.chars().count() > 20 {
                        base_name.chars().take(20).collect()
                    } else {
                        base_name.to_string()
                    };

                    // date is like 2024-05-18T10:00:00.000Z, sanitized for filename
                    let sanitized_date = date.replace(':', "-");
                    let full_filename =
                        format!("img{}_{}.{}", sanitized_date, truncated_base, extension);
                    let file_path = target_dir.join(full_filename);

                    tokio::fs::write(file_path, bytes).await?;
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
                .await;
                (url, result)
            });
        }

        while let Some(res) = set.join_next().await {
            let (url, result) = res.unwrap(); // JoinError
            if let Err(e) = result {
                errors.push(format!("Failed to download {}: {}", url, e));
            }

            downloaded_count += 1;
            self.post_to_ui(RedskyUiMsg::DownloadProgress {
                id,
                processed_posts: all_posts.len(),
                total_posts: Some(all_posts.len()),
                downloaded_images: downloaded_count,
                total_images: Some(total_images),
                status: DownloadStatus::Downloading,
            });
        }

        self.post_to_ui(RedskyUiMsg::DownloadFinished { id, errors });

        Ok(RedskyUiMsg::ActionSucceeded()) // dummy successful msg
    }

    async fn post(
        &self,
        msg: &String,
        image_paths: &Vec<String>,
        reply_to: &Option<(StrongRef, StrongRef)>,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        dbg!("post");

        let mut embed = None;
        if !image_paths.is_empty() {
            let mut images = Vec::new();
            for path in image_paths {
                if let Ok(file_bytes) = tokio::fs::read(path).await {
                    let blob_output = self.bsky_agent.api.com.atproto.repo.upload_blob(file_bytes).await?;
                    images.push(atrium_api::app::bsky::embed::images::ImageData {
                        alt: String::new(),
                        aspect_ratio: None,
                        image: blob_output.blob.clone(),
                    }.into());
                }
            }
            if !images.is_empty() {
                let main_embed = atrium_api::app::bsky::embed::images::MainData { images }.into();
                embed = Some(atrium_api::types::Union::Refs(atrium_api::app::bsky::feed::post::RecordEmbedRefs::AppBskyEmbedImagesMain(Box::new(main_embed))));
            }
        }

        let reply = if let Some((root_ref, parent_ref)) = reply_to {
            let root = atrium_api::com::atproto::repo::strong_ref::MainData {
                cid: root_ref.cid.clone(),
                uri: root_ref.uri.clone(),
            }
            .into();
            let parent = atrium_api::com::atproto::repo::strong_ref::MainData {
                cid: parent_ref.cid.clone(),
                uri: parent_ref.uri.clone(),
            }
            .into();
            Some(atrium_api::app::bsky::feed::post::ReplyRefData { root, parent }.into())
        } else {
            None
        };

        let _ = self
            .bsky_agent
            .create_record(atrium_api::app::bsky::feed::post::RecordData {
                created_at: Datetime::now(),
                embed,
                entities: None,
                facets: None,
                labels: None,
                langs: None,
                reply,
                tags: None,
                text: msg.to_string(),
            })
            .await?;
        Ok(RedskyUiMsg::ActionSucceeded())
    }

    async fn get_unread_count(
        &self,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .notification
            .get_unread_count(
                atrium_api::app::bsky::notification::get_unread_count::ParametersData {
                    priority: None,
                    seen_at: None,
                }
                .into(),
            )
            .await?;

        let count = response.data.count;
        Ok(RedskyUiMsg::NotifyUnreadCount { count })
    }

    async fn get_notifications(
        &self,
    ) -> Result<RedskyUiMsg, Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .bsky_agent
            .api
            .app
            .bsky
            .notification
            .list_notifications(
                atrium_api::app::bsky::notification::list_notifications::ParametersData {
                    cursor: None,
                    limit: Some(30.try_into().unwrap()),
                    priority: None,
                    reasons: None,
                    seen_at: None,
                }
                .into(),
            )
            .await?;

        let notifications = response
            .data
            .notifications
            .into_iter()
            .map(|notif| crate::app::AppNotification {
                uri: notif.uri.clone(),
                author: notif.author.handle.to_string(),
                author_avatar: notif.author.avatar.clone().unwrap_or_default(),
                reason: notif.reason.clone(),
                is_read: notif.is_read,
            })
            .collect();

        Ok(RedskyUiMsg::RefreshNotificationsMsg { notifications })
    }
}
