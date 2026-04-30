impl RedskyApp {
    fn process_message(&mut self, ctx: &egui::Context, msg: RedskyUiMsg) {
        match msg {
            RedskyUiMsg::ActionSucceeded() => {
                self.post_message(BskyActorMsg::GetTimeline { cursor: None });
            }
            RedskyUiMsg::RefreshTimelineMsg {
                posts,
                cursor,
                append,
            } => {
                self.request_post_images(&posts);
                let new_items = crate::app::into_feed_items(posts);
                if append {
                    self.timeline.extend(new_items);
                } else {
                    self.timeline = new_items;
                }
                self.timeline_cursor = cursor;
            }
            RedskyUiMsg::RefreshBookmarksMsg { posts, cursor, append } => {
                self.request_post_images(&posts);
                if append {
                    self.bookmarks.extend(posts);
                } else {
                    self.bookmarks = posts;
                }
                self.bookmarks_cursor = cursor;
            }
            RedskyUiMsg::PrepareUserView { username } => {
                self.user_posts.insert(username.clone(), None);
                self.user_likes_posts.insert(username.clone(), None);
                self.post_message(BskyActorMsg::GetUserProfile { username });
            }
            RedskyUiMsg::PrepareImageView { img_uri } => {
                self.image_cache.insert(img_uri.clone(), None);
                self.post_message(BskyActorMsg::LoadImage { url: img_uri });
            }
            RedskyUiMsg::ShowUserProfile { profile } => {
                self.user_infos_cache
                    .insert(profile.handle.clone(), profile);
            }
            RedskyUiMsg::ShowUserPostsMsg {
                username,
                posts,
                cursor,
                append,
            } => {
                self.request_post_images(&posts);
                let new_items = crate::app::into_feed_items(posts);
                if append {
                    if let Some(Some(existing_posts)) = self.user_posts.get_mut(&username) {
                        existing_posts.extend(new_items);
                    }
                } else {
                    self.user_posts.insert(username.clone(), Some(new_items));
                }
                self.user_cursors.insert(username, cursor);
            }
            RedskyUiMsg::ShowUserLikesMsg {
                username,
                posts,
                cursor,
                append,
            } => {
                self.request_post_images(&posts);
                let new_items = crate::app::into_feed_items(posts);
                if append {
                    if let Some(Some(existing_posts)) = self.user_likes_posts.get_mut(&username) {
                        existing_posts.extend(new_items);
                    }
                } else {
                    self.user_likes_posts.insert(username.clone(), Some(new_items));
                }
                self.user_likes_cursors.insert(username, cursor);
            }
            RedskyUiMsg::PrepareThreadView { thread_ref } => {
                self.post_replies_cache.insert(thread_ref.clone(), None);
                self.post_message(BskyActorMsg::GetPostAndReplies {
                    post_ref: thread_ref,
                });
            }
            RedskyUiMsg::CloseThreadView { thread_ref } => {
                self.post_replies_cache.remove(&thread_ref);
            }
            RedskyUiMsg::DropUserPostsMsg { username } => {
                self.user_posts.remove(&username);
                self.user_likes_posts.remove(&username);
            }
            RedskyUiMsg::ShowErrorMsg { error } => {
                print!("error: {}", error);
            }
            RedskyUiMsg::DeletePost { post_uri, post_cid } => {
                self.post_message(BskyActorMsg::DeletePost { post_uri, post_cid });
            }
            RedskyUiMsg::ShowRawPostView { post_uri, raw_json } => {
                self.opened_raw_views.insert(post_uri, raw_json);
            }
            RedskyUiMsg::CloseRawPostView { post_uri } => {
                self.opened_raw_views.remove(&post_uri);
            }
            RedskyUiMsg::NotifyLikesLoaded { post_uri, likers, cursor, append } => {
                if append {
                    if let Some((existing, existing_cursor)) = self.post_likers_cache.get_mut(&post_uri) {
                        existing.extend(likers);
                        *existing_cursor = cursor;
                    }
                } else {
                    self.post_likers_cache.insert(post_uri, (likers, cursor));
                }
            }
            RedskyUiMsg::NotifyRepostersLoaded {
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
            }
            RedskyUiMsg::CloseLikesView { post_uri } => {
                self.post_likers_cache.remove(&post_uri);
            }
            RedskyUiMsg::CloseRepostersView { post_uri } => {
                self.post_reposters_cache.remove(&post_uri);
            }
            RedskyUiMsg::PrepareLikeAction {
                post_uri,
                post_cid,
                unlike,
            } => {
                if unlike {
                    let mut like_record_uri = String::new();
                    self.update_post_optimistically(&post_uri, |post| {
                        if let Some(uri) = &post.viewer_like {
                            like_record_uri = uri.clone();
                        }
                        post.viewer_like = None;
                        post.like_count = (post.like_count - 1).max(0);
                    });
                    if !like_record_uri.is_empty() {
                        self.post_message(BskyActorMsg::Unlike {
                            post_uri,
                            like_record_uri,
                        });
                    }
                } else {
                    self.update_post_optimistically(&post_uri, |post| {
                        post.viewer_like = Some("pending".to_string());
                        post.like_count += 1;
                    });
                    self.post_message(BskyActorMsg::Like {
                        post_ref: StrongRef {
                            uri: post_uri,
                            cid: post_cid,
                        },
                    });
                }
            }
            RedskyUiMsg::PrepareRepostAction {
                post_uri,
                post_cid,
                unrepost,
            } => {
                if unrepost {
                    let mut repost_record_uri = String::new();
                    self.update_post_optimistically(&post_uri, |post| {
                        if let Some(uri) = &post.viewer_repost {
                            repost_record_uri = uri.clone();
                        }
                        post.viewer_repost = None;
                        post.repost_count = (post.repost_count - 1).max(0);
                    });
                    if !repost_record_uri.is_empty() {
                        self.post_message(BskyActorMsg::Unrepost {
                            post_uri,
                            repost_record_uri,
                        });
                    }
                } else {
                    self.update_post_optimistically(&post_uri, |post| {
                        post.viewer_repost = Some("pending".to_string());
                        post.repost_count += 1;
                    });
                    self.post_message(BskyActorMsg::Repost {
                        post_ref: StrongRef {
                            uri: post_uri,
                            cid: post_cid,
                        },
                    });
                }
            }
            RedskyUiMsg::NotifyLikeActionSucceeded { post_uri, like_uri } => {
                self.update_post_optimistically(&post_uri, |post| {
                    post.viewer_like = Some(like_uri.clone());
                });
            }
            RedskyUiMsg::NotifyRepostActionSucceeded {
                post_uri,
                repost_uri,
            } => {
                self.update_post_optimistically(&post_uri, |post| {
                    post.viewer_repost = Some(repost_uri.clone());
                });
            }
            RedskyUiMsg::NotifyPostAndRepliesLoaded { post, replies } => {
                let strong_ref = StrongRef {
                    uri: post.uri.clone(),
                    cid: post.cid.clone(),
                };
                self.request_post_images(&replies);
                self.request_post_images(&vec![post.clone()]);
                let mut items = vec![FeedItem::Full(post, None)];
                items.extend(replies.into_iter().map(|p| FeedItem::Full(p, None)));
                self.post_replies_cache.insert(strong_ref, Some(items));
            }
            RedskyUiMsg::LogInSucceededMsg() => {
                self.is_logged_in = true;

                if self.remember_me {
                    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
                        let _ = entry.set_password(&format!("{}:{}", self.login, self.pass));
                    }
                } else {
                    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
                        let _ = entry.delete_credential();
                    }
                }

                self.main_view_state = MainViewState::OwnPostFeed;
                self.post_message(BskyActorMsg::GetUserPosts {
                    username: self.login.clone(),
                    cursor: None,
                });
                self.post_message(BskyActorMsg::GetUserProfile {
                    username: self.login.clone(),
                });
                self.post_message(BskyActorMsg::GetTimeline { cursor: None });
                self.post_message(BskyActorMsg::GetBookmarks { cursor: None });
                self.post_message(BskyActorMsg::GetUnreadCount());
                self.post_message(BskyActorMsg::GetNotifications { cursor: None });
            }
            RedskyUiMsg::NotifyUnreadCount { count } => {
                self.unread_notifications = count;
            }
            RedskyUiMsg::RefreshNotificationsMsg { notifications, cursor, append } => {
                for notif in &notifications {
                    self.request_image(&notif.author_avatar);
                }
                if append {
                    self.notifications.extend(notifications);
                } else {
                    self.notifications = notifications;
                }
                self.notifications_cursor = cursor;
            }
            RedskyUiMsg::NotifyImageLoaded { url, data } => {
                let texture = ctx.load_texture(&url, data, Default::default());
                self.image_cache.insert(url.clone(), Some(texture));
                println!("image {} loaded", url);
            }
            RedskyUiMsg::ShowBigImageView { img_uri } => {
                self.opened_image_views.insert(img_uri.clone());
                self.post_message(BskyActorMsg::LoadImage { url: img_uri });
            }
            RedskyUiMsg::CloseBigImageView { img_uri } => {
                self.opened_image_views.remove(&img_uri);
            }
            RedskyUiMsg::DownloadProgress {
                id,
                processed_posts,
                total_posts,
                downloaded_images,
                total_images,
                status,
            } => {
                if let Some(task) = self.download_tasks.get_mut(&id) {
                    task.processed_posts = processed_posts;
                    task.total_posts = total_posts;
                    task.downloaded_images = downloaded_images;
                    task.total_images = total_images;
                    task.status = status;
                }
            }
            RedskyUiMsg::DownloadFinished { id, errors } => {
                if let Some(task) = self.download_tasks.get_mut(&id) {
                    task.status = DownloadStatus::Finished;
                    task.errors = errors;
                }
            }
            RedskyUiMsg::StartDownloadJob { username, path } => {
                let id = self.next_download_id;
                self.next_download_id += 1;
                self.download_tasks.insert(
                    id,
                    DownloadTask {
                        id,
                        username: username.clone(),
                        path: path.clone(),
                        processed_posts: 0,
                        total_posts: None,
                        downloaded_images: 0,
                        total_images: None,
                        status: DownloadStatus::Scanning,
                        errors: Vec::new(),
                    },
                );
                self.post_message(BskyActorMsg::StartImageDownload { id, username, path });
            }
            RedskyUiMsg::ShowSearchResults { results } => {
                for profile in &results {
                    self.request_image(&profile.avatar_uri);
                }
                self.search_results = results;
            }
            RedskyUiMsg::ShowSearchPostsResults { posts, cursor, append } => {
                self.request_post_images(&posts);
                let new_items = crate::app::into_feed_items(posts);
                if append {
                    if let Some(existing_posts) = self.search_posts_results.as_mut() {
                        existing_posts.extend(new_items);
                    }
                } else {
                    self.search_posts_results = Some(new_items);
                }
                self.search_posts_cursor = cursor;
            }
        }
    }
}
